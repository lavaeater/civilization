use bevy::prelude::{Commands, Entity, EventReader, Has, Query, Without};
use bevy::utils::HashMap;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, BuildCityMove, Move, MovementMove, PopExpMove};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::general::general_components::{PlayerAreas, Population, PlayerStock, LandPassage, Token, CitySite};
use crate::civilization::movement::movement_components::TokenHasMoved;

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &PlayerStock)>,
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
                    moves.insert(command_index, Move::PopulationExpansion(PopExpMove::new(
                        *area,
                        pop.max_expansion_for_player(event.player).min(stock.tokens_in_stock()),
                    )));
                }
            }
        }
        commands.entity(event.player).insert(AvailableMoves::new(moves));
    }
}

pub fn recalculate_movement_moves_for_player(
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    player_move_query: Query<&PlayerAreas>,
    area_connections_query: Query<&LandPassage>,
    token_filter_query: Query<&Token, Without<TokenHasMoved>>,
    mut commands: Commands,
) {
    /*
    It's easier now that we have access to all the player's areas in
    the PlayerAreas component. We can just iterate over them and when we
     */
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok(player_areas) = player_move_query.get(event.player) {
            for (area, token_count) in player_areas
                .areas_and_population()
                .iter()
                .map(
                    |(a, p)| 
                        (a.clone(), p
                            .iter()
                            .filter(|t| token_filter_query.contains(**t)).count()))
                .collect::<HashMap<Entity, usize>>() { 
                if let Ok(connections) = area_connections_query.get(area) {
                    for connection in connections.to_areas.iter() {
                        command_index += 1;
                        moves.insert(command_index, Move::Movement(MovementMove::new(
                            area,
                            connection.clone(),
                            event.player,
                            token_count
                        )));
                    }
                }
            }
        }
        moves.insert(command_index + 1, Move::EndMovement);
        commands.entity(event.player).insert(AvailableMoves::new(moves));
    }
}

pub fn recalculate_city_construction_moves_for_player(
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    player_move_query: Query<&PlayerAreas>,
    area_property_query: Query<(&Population, Has<CitySite>)>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        let mut command_index = 0;
        if let Ok(player_areas) = player_move_query.get(event.player) {
            for (area, population) in player_areas.areas_and_population_count().iter() {
                if population >= &6 {
                    if let Ok((_area_pop, has_city_site)) = area_property_query.get(*area) {
                        if has_city_site && population >= &6{
                            command_index += 1;
                            moves.insert(command_index, Move::CityConstruction(BuildCityMove::new(*area, event.player)));
                        } else if population >= &12 {
                            command_index += 1;
                            moves.insert(command_index, Move::CityConstruction(BuildCityMove::new(*area, event.player)));
                        }
                    }
                }
            }
            
            commands.entity(event.player).insert(AvailableMoves::new(moves));
        }
    }
}