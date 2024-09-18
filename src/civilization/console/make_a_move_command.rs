use bevy_console::ConsoleCommand;
use bevy::prelude::{Entity, EventWriter, Query};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::city_construction::city_construction_events::{BuildCity, EndPlayerCityConstruction};
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use crate::civilization::population_expansion::population_expansion_events::ExpandPopulationManuallyCommand;
use crate::civilization::movement::movement_events::{MoveTokenFromAreaToAreaCommand, PlayerMovementEnded};

#[derive(Parser, ConsoleCommand)]
#[command(name = "move")]
pub struct MakeAMove {
    pub player: String,
    pub index: usize,
    pub number: Option<usize>,
}

pub fn make_a_move(
    mut command: ConsoleCommand<MakeAMove>,
    available_moves: Query<(&Name, &AvailableMoves, Entity)>,
    mut expand_writer: EventWriter<ExpandPopulationManuallyCommand>,
    mut move_tokens_writer: EventWriter<MoveTokenFromAreaToAreaCommand>,
    mut end_movement_writer: EventWriter<PlayerMovementEnded>,
    mut build_city_writer: EventWriter<BuildCity>,
    mut end_player_city_construction: EventWriter<EndPlayerCityConstruction>
) {
    if let Some(Ok(MakeAMove { player, index, number })) = command.take() {
        for (name, avail_moves, player_entity) in available_moves.iter() {
            if name.to_string() == player {
                avail_moves.moves.iter().for_each(|(move_index, game_move)| {
                    if index.eq(move_index) {
                        match game_move {
                            Move::PopulationExpansion(pop_exp_move) => {
                                expand_writer.send(ExpandPopulationManuallyCommand::new(player_entity, pop_exp_move.area, number.unwrap_or(pop_exp_move.max_tokens)));
                            }
                            Move::Movement(movement_move) => { 
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, number.unwrap_or(movement_move.max_tokens), player_entity));
                            }
                            Move::EndMovement => {
                                end_movement_writer.send(PlayerMovementEnded::default());
                            }
                            Move::CityConstruction(build_city_move) => {
                                build_city_writer.send(BuildCity::new(player_entity, build_city_move.target));
                            }
                            Move::EndCityConstruction => {
                                end_player_city_construction.send(EndPlayerCityConstruction::new(player_entity));
                            }
                        }
                    }
                });
            }
        }
    }
}
