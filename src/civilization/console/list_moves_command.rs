use bevy_console::ConsoleCommand;
use bevy::prelude::{Query, Res};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use crate::civilization::movement::movement_components::MoveableTokens;

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

            // let message = avail_moves.map(|(from_name, number_of_tokens, targets)| {
            //     format!("{from_name} can move max {number_of_tokens} to: {:?}", targets.iter().map(|name| name.as_str()).collect::<Vec<&str>>().join(", "))
            // }).collect::<Vec<String>>().join("\n");
            command.reply("Moves:".to_string());
        }
    }
}
