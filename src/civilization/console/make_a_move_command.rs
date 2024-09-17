use bevy_console::ConsoleCommand;
use bevy::prelude::Query;
use bevy::core::Name;
use clap::Parser;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};

#[derive(Parser, ConsoleCommand)]
#[command(name = "move")]
pub struct MakeAMove {
    pub player: String,
    pub index: usize,
    pub number: Option<usize>,
}

pub fn make_a_move(
    mut command: ConsoleCommand<MakeAMove>,
    available_moves: Query<(&Name, &AvailableMoves)>,
) {
    if let Some(Ok(MakeAMove { player, index, number })) = command.take() {
        for (name, avail_moves) in available_moves.iter() {
            if name.to_string() == player {
                avail_moves.moves.iter().for_each(|(move_index, game_move)| {
                    if index.eq(move_index) {
                        match game_move {
                            Move::PopulationExpansion(area, max_tokens) => {}
                        }
                    }
                });
            }
        }
    }
}