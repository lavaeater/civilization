use crate::civilization::{
    AvailableMoves, BuildCityMove, BuiltCity, CitySite, CityTokenStock, CivCardName,
    EliminateCityMove, ExpandAutomatically, ExpandManually, GameMove, HasTooManyCities, IsBuilding,
    LandPassage, MovementMove, NeedsExpansion, OpenSea, PlayerAreas, PlayerCities,
    PlayerCivilizationCards, PlayerMovementEnded, PlayerShips, PopExpMove, Population,
    RecalculatePlayerMoves, SeaPassage, TokenHasMoved, TokenStock,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Has, MessageReader, MessageWriter, Name, Query};

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &TokenStock, Option<&PlayerCivilizationCards>)>,
    area_population_query: Query<&Population>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, stock, civ_cards)) = player_move_query.get(event.player) {
            let has_agriculture = civ_cards
                .is_some_and(|c| c.owns(&CivCardName::Agriculture));
            for area in &player_areas.areas() {
                if let Ok(pop) = area_population_query.get(*area) {
                    command_index += 1;
                    moves.insert(
                        command_index,
                        GameMove::PopulationExpansion(PopExpMove::new(
                            *area,
                            pop.max_expansion_for_player_with_agriculture(
                                event.player,
                                has_agriculture,
                            )
                            .min(stock.tokens_in_stock()),
                        )),
                    );
                }
            }
        }
        if moves.is_empty() {
            commands.entity(event.player).remove::<NeedsExpansion>();
            commands.entity(event.player).remove::<ExpandManually>();
            commands
                .entity(event.player)
                .remove::<ExpandAutomatically>();
        } else {
            commands
                .entity(event.player)
                .insert(AvailableMoves::new(moves));
        }
    }
}

pub fn recalculate_movement_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &PlayerShips, Option<&PlayerCivilizationCards>)>,
    area_connections_query: Query<&LandPassage>,
    sea_connections_query: Query<&SeaPassage>,
    area_pop_and_city_query: Query<(&Population, Option<&BuiltCity>)>,
    open_sea_query: Query<Has<OpenSea>>,
    token_filter_query: Query<Has<TokenHasMoved>>,
    mut commands: Commands,
    mut end_player_movement: MessageWriter<PlayerMovementEnded>,
    names: Query<&Name>,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, player_ships, civ_cards)) = player_move_query.get(event.player) {
            let has_road_building = civ_cards
                .is_some_and(|c| c.owns(&CivCardName::RoadBuilding));
            let has_astronomy = civ_cards
                .is_some_and(|c| c.owns(&CivCardName::Astronomy));
            let has_cloth_making = civ_cards
                .is_some_and(|c| c.owns(&CivCardName::ClothMaking));

            let areas_and_population = player_areas.areas_and_population();

            for (area, tokens) in &areas_and_population {
                let tokens_that_can_move = tokens
                    .iter()
                    .filter(|t| !token_filter_query.get(**t).unwrap())
                    .collect::<Vec<_>>();

                if !tokens_that_can_move.is_empty() {
                    // ── Land movement ─────────────────────────────────────────
                    if let Ok(connections) = area_connections_query.get(*area) {
                        for target_area in &connections.to_areas {
                            if let Ok((population, optional_city)) =
                                area_pop_and_city_query.get(*target_area)
                            {
                                add_land_move(
                                    &mut moves,
                                    &mut command_index,
                                    *area,
                                    *target_area,
                                    event.player,
                                    tokens_that_can_move.len(),
                                    population,
                                    optional_city,
                                );
                            }

                            // ── Road Building: extend one hop through empty, friendly areas ──
                            // Rule 23.31: may pass through one area that has no enemies and no cities.
                            // Cannot use Road Building to then board a ship.
                            if has_road_building
                && let Ok((pass_pop, None)) =
                area_pop_and_city_query.get(*target_area) &&
                // Through area must be free of enemies and cities
                !pass_pop.has_other_players(&event.player) && let Ok(connections2) =
                    area_connections_query.get(*target_area)
                            {
                                for final_area in &connections2.to_areas {
                                    if *final_area == *area {
                                        continue;
                                    } // no backtrack
                                    if let Ok((final_pop, final_city)) =
                                        area_pop_and_city_query.get(*final_area)
                                    {
                                        add_land_move(
                                            &mut moves,
                                            &mut command_index,
                                            *area,
                                            *final_area,
                                            event.player,
                                            tokens_that_can_move.len(),
                                            final_pop,
                                            final_city,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Ship movement (rule 23.1) ───────────────────────────────────
            // Ships move independently of whether they're carrying tokens --
            // iterate every area with one of the player's ships, not just
            // areas with population, so a ship stranded away from its owner's
            // tokens (or simply left empty) can still move on its own.
            // Astronomy (rule 28.23): allows entering Open Sea areas.
            // Cloth Making (rule 28.18): extends sea range by 1 hop.
            for area in player_ships.all_areas_with_ships() {
                let Ok(sea) = sea_connections_query.get(area) else {
                    continue;
                };
                let ferry_tokens = areas_and_population
                    .get(&area)
                    .map_or(0, |tokens| {
                        tokens
                            .iter()
                            .filter(|t| !token_filter_query.get(**t).unwrap())
                            .count()
                    })
                    .min(5);
                for &target_area in &sea.to_areas {
                    // Without Astronomy, skip Open Sea destinations
                    if !has_astronomy && open_sea_query.get(target_area).unwrap_or(false) {
                        continue;
                    }
                    command_index += 1;
                    moves.insert(
                        command_index,
                        GameMove::ShipFerry(MovementMove::new(
                            area,
                            target_area,
                            event.player,
                            ferry_tokens,
                        )),
                    );

                    // Cloth Making: generate one additional hop through this target
                    // (rule 28.18: +1 sea movement range, cannot use open sea without Astronomy)
                    if has_cloth_making
                        && let Ok(sea2) = sea_connections_query.get(target_area)
                    {
                        for &final_area in &sea2.to_areas {
                            if final_area == area {
                                continue;
                            } // no backtrack
                            if !has_astronomy && open_sea_query.get(final_area).unwrap_or(false) {
                                continue;
                            }
                            command_index += 1;
                            moves.insert(
                                command_index,
                                GameMove::ShipFerry(MovementMove::new(
                                    area,
                                    final_area,
                                    event.player,
                                    ferry_tokens,
                                )),
                            );
                        }
                    }
                }
            }
        }

        let _player_name = names.get(event.player).map_or("?", bevy::prelude::Name::as_str);
        if moves.is_empty() {
            end_player_movement.write(PlayerMovementEnded::new(event.player));
        } else {
            moves.insert(command_index + 1, GameMove::EndMovement);
            commands
                .entity(event.player)
                .insert(AvailableMoves::new(moves));
        }
    }
}

/// Classify a land move and insert it into the moves map.
fn add_land_move(
    moves: &mut HashMap<usize, GameMove>,
    command_index: &mut usize,
    source: bevy::prelude::Entity,
    target: bevy::prelude::Entity,
    player: bevy::prelude::Entity,
    token_count: usize,
    target_pop: &Population,
    target_city: Option<&BuiltCity>,
) {
    *command_index += 1;
    let game_move = if let Some(city) = target_city {
        if city.player == player {
            // Player's own city – don't generate a move into it
            *command_index -= 1;
            return;
        }
        GameMove::AttackCity(MovementMove::new(source, target, player, token_count))
    } else if target_pop.has_other_players(&player) {
        GameMove::AttackArea(MovementMove::new(source, target, player, token_count))
    } else {
        GameMove::Movement(MovementMove::new(source, target, player, token_count))
    };
    moves.insert(*command_index, game_move);
}

pub fn recalculate_city_construction_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(
        &PlayerAreas,
        &CityTokenStock,
        Option<&PlayerCivilizationCards>,
    )>,
    area_property_query: Query<(&Population, Has<CitySite>)>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, city_token_stock, civ_cards)) = player_move_query.get(event.player)
        {
            // Architecture (rule 25.3): holder can build with 1 fewer population token
            // (minimum 1); the saved token goes to treasury.
            let has_architecture = civ_cards
                .is_some_and(|c| c.owns(&CivCardName::Architecture));
            let city_site_threshold: usize = if has_architecture { 5 } else { 6 };
            let no_site_threshold: usize = if has_architecture { 11 } else { 12 };

            if city_token_stock.has_tokens() {
                for (area, population) in &player_areas.areas_and_population_count() {
                    if let Ok((_area_pop, has_city_site)) = area_property_query.get(*area)
                        && ((has_city_site && *population >= city_site_threshold)
                            || *population >= no_site_threshold)
                    {
                        command_index += 1;
                        moves.insert(
                            command_index,
                            GameMove::CityConstruction(BuildCityMove::new(*area, event.player)),
                        );
                    }
                }
            }
            if moves.is_empty() {
                commands.entity(event.player).remove::<IsBuilding>();
            } else {
                command_index += 1;
                moves.insert(command_index, GameMove::EndCityConstruction);
                commands
                    .entity(event.player)
                    .insert(AvailableMoves::new(moves));
            }
        }
    }
}

pub fn recalculate_city_support_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_city_query: Query<(&PlayerCities, &HasTooManyCities)>,
    area_property_query: Query<&Population>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_cities, has_too_many_cities)) = player_city_query.get(event.player) {
            for (area, city) in &player_cities.areas_and_cities {
                if let Ok(pop) = area_property_query.get(*area) {
                    command_index += 1;
                    moves.insert(
                        command_index,
                        GameMove::EliminateCity(EliminateCityMove::new(
                            event.player,
                            *area,
                            *city,
                            pop.max_population,
                            has_too_many_cities.needed_tokens,
                        )),
                    );
                }
            }
            if moves.is_empty() {
                commands.entity(event.player).remove::<HasTooManyCities>();
            } else {
                commands
                    .entity(event.player)
                    .insert(AvailableMoves::new(moves));
            }
        }
    }
}
