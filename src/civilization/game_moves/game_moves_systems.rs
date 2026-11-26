use crate::civilization::{
    AvailableMoves, BuildCityMove, BuiltCity, CitySite, CityTokenStock, CivCardName,
    EliminateCityMove, ExpandAutomatically, ExpandManually, GameMove, HasTooManyCities, IsBuilding,
    LandPassage, MovementMove, NeedsExpansion, OpenSea, PlayerAreas, PlayerCities,
    PlayerCivilizationCards, PlayerMovementEnded, PlayerShips, PopExpMove, Population,
    RecalculatePlayerMoves, SHIP_CAPACITY, SeaPassage, ShipVoyages, TokenHasMoved, TokenStock,
    ship_range,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Entity, Has, MessageReader, MessageWriter, Name, Query, Res, info};

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(
        &PlayerAreas,
        &TokenStock,
        Option<&PlayerCivilizationCards>,
        Option<&NeedsExpansion>,
    )>,
    area_population_query: Query<(&Population, Has<BuiltCity>)>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, stock, civ_cards, Some(needs_expansion))) =
            player_move_query.get(event.player)
        {
            let has_agriculture = civ_cards.is_some_and(|c| c.owns(&CivCardName::Agriculture));
            // Only offer areas still pending this round's expansion --
            // `player_areas.areas()` alone doesn't shrink once an area is
            // expanded, and `max_expansion_for_player_with_agriculture` is
            // based on current token count, not "already used this round",
            // so re-offering an already-done area would let it be expanded
            // more than once per round.
            for area in &player_areas.areas() {
                if !needs_expansion.areas_that_need_expansion.contains(area) {
                    continue;
                }
                if let Ok((pop, has_city)) = area_population_query.get(*area) {
                    let max_tokens = pop
                        .max_expansion_for_player_with_agriculture(event.player, has_agriculture)
                        .min(stock.tokens_in_stock());
                    // No growth in a city area, and nothing to offer at 0.
                    if has_city || max_tokens == 0 {
                        continue;
                    }
                    command_index += 1;
                    moves.insert(
                        command_index,
                        GameMove::PopulationExpansion(PopExpMove::new(*area, max_tokens)),
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
    voyages: Option<Res<ShipVoyages>>,
) {
    let no_voyages = ShipVoyages::default();
    let voyages = voyages.as_deref().unwrap_or(&no_voyages);
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, player_ships, civ_cards)) = player_move_query.get(event.player) {
            let has_road_building = civ_cards.is_some_and(|c| c.owns(&CivCardName::RoadBuilding));
            let has_astronomy = civ_cards.is_some_and(|c| c.owns(&CivCardName::Astronomy));
            let has_cloth_making = civ_cards.is_some_and(|c| c.owns(&CivCardName::ClothMaking));

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

            // ── Ship movement (rules 23.1, 23.5) ─────────────────────────────
            // Ships move whether or not they carry anything, so every area
            // with one of the player's ships is a source. A ship may enter up
            // to four water areas a phase (five with Cloth Making) over any
            // number of trips, picking up and dropping off tokens on the way.
            // One move sails one ship to a coast within its remaining range;
            // `max_tokens` is how many more tokens it can take aboard here.
            let range = ship_range(has_cloth_making);
            for area in player_ships.all_areas_with_ships() {
                let ships = player_ships.ships_in_area(area);
                let range_left = voyages.best_range_left(ships, range);
                if range_left == 0 {
                    continue;
                }
                let unmoved = areas_and_population.get(&area).map_or(0, |tokens| {
                    tokens
                        .iter()
                        .filter(|t| !token_filter_query.get(**t).unwrap())
                        .count()
                });
                for (target_area, cost) in ship_routes(
                    area,
                    range_left,
                    has_astronomy,
                    &sea_connections_query,
                    &open_sea_query,
                ) {
                    let Some(ship) = voyages.pick_ship(ships, range, cost) else {
                        continue;
                    };
                    let room = SHIP_CAPACITY.saturating_sub(voyages.get(ship).cargo.len());
                    command_index += 1;
                    moves.insert(
                        command_index,
                        GameMove::ShipFerry(MovementMove::new(
                            area,
                            target_area,
                            event.player,
                            unmoved.min(room),
                        )),
                    );
                }
            }
        }

        let _player_name = names
            .get(event.player)
            .map_or("?", bevy::prelude::Name::as_str);
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
        Option<&Name>,
    )>,
    area_property_query: Query<(&Population, Has<CitySite>, Has<BuiltCity>)>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, city_token_stock, civ_cards, name)) =
            player_move_query.get(event.player)
        {
            // Architecture (rule 25.3): holder can build with 1 fewer population token
            // (minimum 1); the saved token goes to treasury.
            let has_architecture = civ_cards.is_some_and(|c| c.owns(&CivCardName::Architecture));
            let city_site_threshold: usize = if has_architecture { 5 } else { 6 };
            let no_site_threshold: usize = if has_architecture { 11 } else { 12 };

            if city_token_stock.has_tokens() {
                // Check against the real board state (`Population`), not
                // `PlayerAreas`'s cached mirror -- the two can drift out of
                // sync (see `log_round_state`'s DESYNC diagnostic), and a
                // stale/low cached count here would silently deny a player
                // their build option even though they have enough tokens on
                // the board, leaving the (real) surplus to be stripped away
                // in RemoveSurplusPopulation instead of spent on a city.
                for area in player_areas.areas() {
                    if let Ok((population, has_city_site, false)) = area_property_query.get(area) {
                        // Rule 25.1: one city per area.
                        let pop_count = population.population_for_player(event.player);
                        if (has_city_site && pop_count >= city_site_threshold)
                            || pop_count >= no_site_threshold
                        {
                            command_index += 1;
                            moves.insert(
                                command_index,
                                GameMove::CityConstruction(BuildCityMove::new(area, event.player)),
                            );
                        }
                    }
                }
            }
            if moves.is_empty() {
                // Diagnostic for "why did this player never build?": the best
                // concentration they reached on any city site (needs
                // `city_site_threshold`) and on any area at all.
                let mut best_site = 0;
                let mut best_any = 0;
                for area in player_areas.areas() {
                    if let Ok((population, has_city_site, false)) = area_property_query.get(area) {
                        let pop_count = population.population_for_player(event.player);
                        best_any = best_any.max(pop_count);
                        if has_city_site {
                            best_site = best_site.max(pop_count);
                        }
                    }
                }
                info!(
                    "[CITY_CONSTRUCTION] No build moves for {} (city tokens in stock: {}, areas: {}, best city-site stack: {}/{}, best stack anywhere: {}) - removing IsBuilding",
                    name.map_or_else(
                        || format!("{:?}", event.player),
                        std::string::ToString::to_string
                    ),
                    city_token_stock.city_tokens_in_stock(),
                    player_areas.areas().len(),
                    best_site,
                    city_site_threshold,
                    best_any
                );
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

/// Coasts a ship at `from` can reach by entering at most `max_areas` water
/// areas, with how many it enters to get there (fewest first). Each step along
/// a sea passage enters one area. Open sea may only be crossed with Astronomy
/// (rule 23.54) and is never a place to stop (23.55).
pub fn ship_routes(
    from: Entity,
    max_areas: usize,
    has_astronomy: bool,
    sea: &Query<&SeaPassage>,
    open_sea: &Query<Has<OpenSea>>,
) -> Vec<(Entity, usize)> {
    let is_open = |e: Entity| open_sea.get(e).unwrap_or(false);
    let mut cost: HashMap<Entity, usize> = HashMap::default();
    cost.insert(from, 0);
    let mut frontier = vec![from];
    let mut routes = Vec::new();
    for step in 1..=max_areas {
        let mut next_frontier = Vec::new();
        for at in frontier {
            let Ok(passage) = sea.get(at) else {
                continue;
            };
            for &next in &passage.to_areas {
                if cost.contains_key(&next) || (is_open(next) && !has_astronomy) {
                    continue;
                }
                cost.insert(next, step);
                next_frontier.push(next);
                if !is_open(next) {
                    routes.push((next, step));
                }
            }
        }
        frontier = next_frontier;
    }
    routes
}

#[cfg(test)]
mod open_sea_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::World;

    /// Coast A - coast B along the shore; A and C both border open sea S;
    /// C - D along the shore.
    fn world() -> (World, [Entity; 5]) {
        let mut world = World::new();
        let [a, b, c, d, s] = [(); 5].map(|()| world.spawn(SeaPassage::default()).id());
        world.entity_mut(s).insert(OpenSea);
        for (from, to) in [(a, b), (a, s), (c, s), (c, d)] {
            world.get_mut::<SeaPassage>(from).unwrap().add_passage(to);
            world.get_mut::<SeaPassage>(to).unwrap().add_passage(from);
        }
        (world, [a, b, c, d, s])
    }

    fn routes(
        world: &mut World,
        from: Entity,
        max: usize,
        astronomy: bool,
    ) -> Vec<(Entity, usize)> {
        world
            .run_system_once(move |sea: Query<&SeaPassage>, open: Query<Has<OpenSea>>| {
                ship_routes(from, max, astronomy, &sea, &open)
            })
            .unwrap()
    }

    #[test]
    fn open_sea_needs_astronomy_and_every_area_counts() {
        let (mut world, [a, b, c, d, s]) = world();
        // Without Astronomy: coast only.
        assert_eq!(routes(&mut world, a, 4, false), vec![(b, 1)]);
        // With it: across the open sea (two areas), never stopping in it.
        let with = routes(&mut world, a, 4, true);
        assert_eq!(with, vec![(b, 1), (c, 2), (d, 3)]);
        assert!(with.iter().all(|(e, _)| *e != s));
        // Range limits the trip.
        assert_eq!(routes(&mut world, a, 2, true), vec![(b, 1), (c, 2)]);
        assert_eq!(routes(&mut world, a, 0, true), vec![]);
    }
}
