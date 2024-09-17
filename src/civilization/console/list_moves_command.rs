use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use bevy::core::Name;
use bevy::prelude::Query;
use bevy_console::ConsoleCommand;
use clap::Parser;

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
                    Move::PopulationExpansion(area, max_tokens) => {
                        let area_name = name_query.get(*area).unwrap();
                        command.reply(format!("Move {} - Population Expansion in area {} with max tokens {}", index, area_name, max_tokens));
                    }
                }
            });
        }
    }
}
