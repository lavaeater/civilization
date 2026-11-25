use crate::civilization::{
    AvailableMoves, BuildCityMove, BuiltCity, CitySite, CityTokenStock, CivCardName,
    EliminateCityMove, ExpandAutomatically, ExpandManually, GameMove, HasTooManyCities, IsBuilding,
    LandPassage, MovementMove, NeedsExpansion, OpenSea, PlayerAreas, PlayerCities,
    PlayerCivilizationCards, PlayerMovementEnded, PlayerShips, PopExpMove, Population,
    RecalculatePlayerMoves, SeaPassage, TokenHasMoved, TokenStock,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Entity, Has, MessageReader, MessageWriter, Name, Query, info};

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(
        &PlayerAreas,
        &TokenStock,
        Option<&PlayerCivilizationCards>,
        Option<&NeedsExpansion>,
    )>,
    area_population_query: Query<&Population>,
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

            // ── Ship movement (rule 23.1) ───────────────────────────────────
            // Ships move independently of whether they're carrying tokens --
            // iterate every area with one of the player's ships, not just
            // areas with population, so a ship stranded away from its owner's
            // tokens (or simply left empty) can still move on its own.
            // Astronomy (rule 28.23): allows entering Open Sea areas.
            // Cloth Making (rule 28.18): extends sea range by 1 hop.
            for area in player_ships.all_areas_with_ships() {
                let ferry_tokens = areas_and_population
                    .get(&area)
                    .map_or(0, |tokens| {
                        tokens
                            .iter()
                            .filter(|t| !token_filter_query.get(**t).unwrap())
                            .count()
                    })
                    .min(5);
                for target_area in ship_destinations(
                    area,
                    has_astronomy,
                    has_cloth_making,
                    &sea_connections_query,
                    &open_sea_query,
                ) {
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

/// Coastal areas a ship in `from` can reach this move.
///
/// - One sea hop along the coast.
/// - Cloth Making (rule 28.18): one more coastal hop, no backtracking.
/// - Open sea (rule 23.52) needs Astronomy. A ship never stops in it: entering
///   an open sea means crossing it, so its destinations are the other coasts
///   of that sea, in the same move.
pub fn ship_destinations(
    from: Entity,
    has_astronomy: bool,
    has_cloth_making: bool,
    sea: &Query<&SeaPassage>,
    open_sea: &Query<Has<OpenSea>>,
) -> Vec<Entity> {
    let is_open = |e: Entity| open_sea.get(e).unwrap_or(false);
    // Coasts reachable by one hop: direct coastal links, plus (with
    // Astronomy) every far shore of an adjacent open sea.
    let one_hop = |at: Entity| -> Vec<Entity> {
        let mut out = Vec::new();
        let Ok(passage) = sea.get(at) else {
            return out;
        };
        for &next in &passage.to_areas {
            if !is_open(next) {
                out.push(next);
            } else if has_astronomy && let Ok(open) = sea.get(next) {
                out.extend(open.to_areas.iter().copied().filter(|&c| !is_open(c)));
            }
        }
        out
    };

    let mut destinations: Vec<Entity> = Vec::new();
    let push = |e: Entity, list: &mut Vec<Entity>| {
        if e != from && !list.contains(&e) {
            list.push(e);
        }
    };
    for first in one_hop(from) {
        push(first, &mut destinations);
        if has_cloth_making {
            for second in one_hop(first) {
                push(second, &mut destinations);
            }
        }
    }
    destinations
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

    fn destinations(world: &mut World, from: Entity, astronomy: bool, cloth: bool) -> Vec<Entity> {
        world
            .run_system_once(move |sea: Query<&SeaPassage>, open: Query<Has<OpenSea>>| {
                ship_destinations(from, astronomy, cloth, &sea, &open)
            })
            .unwrap()
    }

    #[test]
    fn open_sea_needs_astronomy_and_is_crossed_in_one_move() {
        let (mut world, [a, b, c, d, s]) = world();
        // Without Astronomy: coast only.
        assert_eq!(destinations(&mut world, a, false, false), vec![b]);
        // With it: straight across to the far shore, never stopping in S.
        let with = destinations(&mut world, a, true, false);
        assert_eq!(with, vec![b, c]);
        assert!(!with.contains(&s));
        // Cloth Making adds one coastal hop after the crossing.
        assert_eq!(destinations(&mut world, a, true, true), vec![b, c, d]);
        assert_eq!(destinations(&mut world, a, false, true), vec![b]);
    }
}
