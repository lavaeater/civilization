use crate::civilization::components::prelude::*;
use crate::civilization::events::prelude::*;
use crate::stupid_ai::stupid_ai_components::StupidAi;
use crate::stupid_ai::stupid_ai_events::{SelectStupidMove, StupidAiEvent};
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
    target_area_info_query: Query<(&Population, Has<BuiltCity>)>,
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
            let available_moves = available_moves.moves.values().filter(|m| 
                match m {
                    Move::Movement(move_ment) => {
                        let (_population, has_city) = target_area_info_query.get(move_ment.target).unwrap();
                        !has_city 
                    }
                    // population.has_player(move_ment.player) && 
                    _ => true,
                }).collect::<Vec<_>>();
           
            let mut rng = rand::thread_rng();
            if let Some(selected_move) = available_moves.into_iter().choose(&mut rng) {
                match selected_move {
                    Move::PopulationExpansion(pop_exp_move) => {
                        expand_writer.send(ExpandPopulationManuallyCommand::new(event.player, pop_exp_move.area, pop_exp_move.max_tokens));
                    }
                    Move::Movement(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, false);
                    }
                    Move::EndMovement => {
                        end_movement_writer.send(PlayerMovementEnded::new(event.player));
                    }
                    Move::CityConstruction(build_city_move) => {
                        build_city_writer.send(BuildCityCommand::new(event.player, build_city_move.target));
                    }
                    Move::EndCityConstruction => {
                        end_player_city_construction.send(EndPlayerCityConstruction::new(event.player));
                    }
                    Move::EliminateCity(el_move) => {
                        eliminate_city.send(EliminateCity::new(el_move.player, el_move.city, el_move.area, false));
                    }
                    Move::AttackArea(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, true);
                    }
                    Move::AttackCity(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        send_movement_move(&mut move_tokens_writer, event, movement_move, true);
                    }
                }
            }
        }
    }
}

fn send_movement_move(move_tokens_writer: &mut EventWriter<MoveTokenFromAreaToAreaCommand>, event: &SelectStupidMove, movement_move: &MovementMove, is_attack: bool) {
    if is_attack {
        move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, movement_move.max_tokens - 1, event.player));
    } else {
        match movement_move.max_tokens {
            1 => {
                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 1, event.player));
            }
            2 => {
                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 1, event.player));
            }
            _ => {
                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 2, event.player));
            }
        }
    }
}