use bevy_console::ConsoleCommand;
use bevy::prelude::{Entity, EventWriter, Query};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use crate::civilization::population_expansion::population_expansion_events::ExpandPopulationManuallyCommand;

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
    mut expand_writer: EventWriter<ExpandPopulationManuallyCommand>
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
                            Move::Movement(_) => { panic!("Cannot make a move of type Movement") }
                            Move::EndMovement => { panic!("Cannot make a move of type EndMovement") }
                        }
                    }
                });
            }
        }
    }
}
