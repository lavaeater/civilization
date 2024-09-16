use bevy_console::ConsoleCommand;
use bevy::prelude::{Query, Res};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};

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

            avail_moves.moves.iter().for_each(|game_move| {
                match game_move {
                    Move::PopulationExpansion(index, area, max_tokens) => {
                        let area_name = name_query.get(*area).unwrap();
                        command.reply(format!("Move {} - Population Expansion in area {} with max tokens {}", index, area_name, max_tokens));
                    }
                }
            });
        }
    }
}
