use crate::civilization::components::population::Population;
use crate::civilization::components::*;
use crate::civilization::concepts::check_city_support::check_city_support_components::HasTooManyCities;
use crate::civilization::concepts::city_construction::city_construction_components::IsBuilding;
use crate::civilization::concepts::movement::movement_components::TokenHasMoved;
use crate::civilization::concepts::movement::movement_events::PlayerMovementEnded;
use crate::civilization::concepts::population_expansion::population_expansion_components::{
    ExpandAutomatically, ExpandManually, NeedsExpansion,
};
use crate::civilization::game_moves::game_moves_components::{
    AvailableMoves, BuildCityMove, EliminateCityMove, Move, MovementMove, PopExpMove,
};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, MessageReader, MessageWriter, Has, Query};

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &TokenStock)>,
    area_population_query: Query<&Population>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        //Finally, we arrive at what we WANT to do!
        /*
        OK, we know which areas we have tokens in. Those are areas
        that
        */
        let mut command_index = 0;
        if let Ok((player_areas, stock)) = player_move_query.get(event.player) {
            for area in player_areas.areas().iter() {
                if let Ok(pop) = area_population_query.get(*area) {
                    command_index += 1;
                    moves.insert(
                        command_index,
                        Move::PopulationExpansion(PopExpMove::new(
                            *area,
                            pop.max_expansion_for_player(event.player)
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
    player_move_query: Query<&PlayerAreas>,
    area_connections_query: Query<&LandPassage>,
    area_pop_and_city_query: Query<(&Population, Option<&BuiltCity>)>,
    token_filter_query: Query<Has<TokenHasMoved>>,
    mut commands: Commands,
    mut end_player_movement: MessageWriter<PlayerMovementEnded>,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok(player_areas) = player_move_query.get(event.player) {
            for (area, tokens) in player_areas.areas_and_population() {
                let tokens_that_can_move = tokens
                    .iter()
                    .filter(|t| !token_filter_query.get(**t).unwrap())
                    .collect::<Vec<_>>();

                if !tokens_that_can_move.is_empty() {
                    if let Ok(connections) = area_connections_query.get(area) {
                        for target_area in connections.to_areas.iter() {
                            if let Ok((population, optional_city)) =
                                area_pop_and_city_query.get(*target_area)
                            {
                                if let Some(has_city) = optional_city {
                                    if has_city.player != event.player {
                                        command_index += 1;
                                        moves.insert(
                                            command_index,
                                            Move::AttackCity(MovementMove::new(
                                                area,
                                                *target_area,
                                                event.player,
                                                tokens_that_can_move.len(),
                                            )),
                                        );
                                    }
                                } else if population.has_other_players(&event.player) {
                                    command_index += 1;
                                    moves.insert(
                                        command_index,
                                        Move::AttackArea(MovementMove::new(
                                            area,
                                            *target_area,
                                            event.player,
                                            tokens_that_can_move.len(),
                                        )),
                                    );
                                } else {
                                    command_index += 1;
                                    moves.insert(
                                        command_index,
                                        Move::Movement(MovementMove::new(
                                            area,
                                            *target_area,
                                            event.player,
                                            tokens_that_can_move.len(),
                                        )),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if moves.is_empty() {
            end_player_movement.write(PlayerMovementEnded::new(event.player));
        } else {
            moves.insert(command_index + 1, Move::EndMovement);
            commands
                .entity(event.player)
                .insert(AvailableMoves::new(moves));
        }
    }
}

pub fn recalculate_city_construction_moves_for_player(
    mut recalc_player_reader: MessageReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &CityTokenStock)>,
    area_property_query: Query<(&Population, Has<CitySite>)>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok((player_areas, city_token_stock)) = player_move_query.get(event.player) {
            if city_token_stock.has_tokens() {
                for (area, population) in player_areas.areas_and_population_count().iter() {
                    if population >= &6 {
                        if let Ok((_area_pop, has_city_site)) = area_property_query.get(*area) {
                            if (has_city_site && population >= &6) || (population >= &12) {
                                command_index += 1;
                                moves.insert(
                                    command_index,
                                    Move::CityConstruction(BuildCityMove::new(*area, event.player)),
                                );
                            }
                        }
                    }
                }
            }
            if moves.is_empty() {
                commands.entity(event.player).remove::<IsBuilding>();
            } else {
                command_index += 1;
                moves.insert(command_index, Move::EndCityConstruction);
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
            for (area, city) in player_cities.areas_and_cities.iter() {
                if let Ok(pop) = area_property_query.get(*area) {
                    command_index += 1;
                    moves.insert(
                        command_index,
                        Move::EliminateCity(EliminateCityMove::new(
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
