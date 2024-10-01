use crate::civilization::city_construction::city_construction_events::{BuildCityCommand, EndPlayerCityConstruction};
use crate::civilization::city_support::city_support_events::EliminateCity;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use crate::civilization::general::general_components::{BuiltCity, PlayerAreas, Population};
use crate::civilization::movement::movement_events::{MoveTokenFromAreaToAreaCommand, PlayerMovementEnded};
use crate::civilization::population_expansion::population_expansion_events::ExpandPopulationManuallyCommand;
use crate::stupid_ai::stupid_ai_components::StupidAi;
use crate::stupid_ai::stupid_ai_events::{SelectStupidMove, StupidAiEvent};
use bevy::log::debug;
use bevy::prelude::{Commands, EventReader, EventWriter, Has, Query};
use rand::prelude::IteratorRandom;

pub fn setup_stupid_ai(
    mut stupid_ai_event: EventReader<StupidAiEvent>,
    mut commands: Commands,
) {
    for e in stupid_ai_event.read() {
        commands.entity(e.player).insert(StupidAi);
    }
}

pub fn select_stupid_move(
    mut event_reader: EventReader<SelectStupidMove>,
    player_moves: Query<(&AvailableMoves, &PlayerAreas)>,
    _area_query: Query<(&Population, Has<BuiltCity>)>,
    mut expand_writer: EventWriter<ExpandPopulationManuallyCommand>,
    mut move_tokens_writer: EventWriter<MoveTokenFromAreaToAreaCommand>,
    mut end_movement_writer: EventWriter<PlayerMovementEnded>,
    mut build_city_writer: EventWriter<BuildCityCommand>,
    mut end_player_city_construction: EventWriter<EndPlayerCityConstruction>,
    mut eliminate_city: EventWriter<EliminateCity>,
) {
    for event in event_reader.read() {
        // debug!("Selecting stupid AI move for player {:?}", event.player);
        if let Ok((available_moves, _player_areas)) = player_moves.get(event.player) {
            /*  
            So, the moves will always really be of maximum one or two types (for now). 
            One is ending the moves, like, not building a city or something like that.
            The other is simply to do something. So, for our first iteration of all this, 
            the stupid stupid AI will always do a random move.
            
            
            Random moves will do for now but won't cut it in the long run - we have to make the non-
            stupid AI make its moves in a more sophisticated manner.
             */
            debug!("Available moves: {:?}", available_moves);

            let mut rng = rand::thread_rng();
            if let Some(selected_move) = available_moves.moves.values().choose(&mut rng) {
                debug!("Selected move: {:?}", selected_move);
                match selected_move {
                    Move::PopulationExpansion(pop_exp_move) => {
                        expand_writer.send(ExpandPopulationManuallyCommand::new(event.player, pop_exp_move.area, pop_exp_move.max_tokens));
                    }
                    Move::Movement(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        match movement_move.max_tokens {
                            1 => {
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 1, event.player));
                            }
                            2 => {
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 1, event.player));
                            }
                            _ => {
                                debug!("This is a move with more than 2 tokens, so we always move two");
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 2, event.player));
                            }
                        }
                    }
                    Move::EndMovement => {
                        debug!("Ending movement for player {:?}", event.player);
                        end_movement_writer.send(PlayerMovementEnded::new(event.player));
                    }
                    Move::CityConstruction(build_city_move) => {
                        debug!("Building city for player {:?}", event.player);
                        build_city_writer.send(BuildCityCommand::new(event.player, build_city_move.target));
                    }
                    Move::EndCityConstruction => {
                        debug!("End City Construction {:?}", event.player);
                        end_player_city_construction.send(EndPlayerCityConstruction::new(event.player));
                    }
                    Move::EliminateCity(el_move) => {
                        debug!("Eliminating city for player {:?}", event.player);
                        eliminate_city.send(EliminateCity::new(el_move.player, el_move.city, el_move.area));
                    }
                }
            }
        }
    }
}