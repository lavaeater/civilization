use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use bevy::core::Name;
use bevy::prelude::Query;
use bevy_console::ConsoleCommand;
use clap::Parser;

#[derive(Parser, ConsoleCommand)]
#[command(name = "moves")]
pub struct ListMoves;

pub fn list_moves(
    mut command: ConsoleCommand<ListMoves>,
    available_moves: Query<(&Name, &AvailableMoves)>,
    name_query: Query<&Name>,
) {
    if let Some(Ok(ListMoves {})) = command.take() {
        for (name, avail_moves) in available_moves.iter() {
            command.reply(format!("Player {} can perform the following moves", name));

            avail_moves.moves.iter().for_each(|(index, game_move)| {
                match game_move {
                    Move::PopulationExpansion(pop_exp_move) => {
                        let area_name = name_query.get(pop_exp_move.area).unwrap();
                        command.reply(format!("Move {} - Population Expansion in area {} with max tokens {}", index, area_name, pop_exp_move.max_tokens));
                    }
                    Move::Movement(_) => {}
                }
            });
        }
    }
}
