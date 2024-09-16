use bevy_console::ConsoleCommand;
use bevy::prelude::{Query, Res};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::movement::movement_components::MoveableTokens;

#[derive(Parser, ConsoleCommand)]
#[command(name = "moves")]
pub struct ListMoves;

pub fn list_moves(
    mut command: ConsoleCommand<ListMoves>,
    moveable_tokens: Query<(&Name, &MoveableTokens)>,
    name_query: Query<&Name>,
    game_info: Res<GameInfoAndStuff>,
) {
    if let Some(Ok(ListMoves {})) = command.take() {
        if let Some(_player_to_move) = game_info.current_mover {
            let moves = moveable_tokens
                .iter()
                .map(|(from_name, move_specs)| {
                    (from_name, move_specs.tokens.iter().count(),
                     move_specs
                         .targets
                         .iter()
                         .map(|target| {
                             let target_name = name_query.get(*target).unwrap();
                             target_name
                         }).collect::<Vec<&Name>>()
                    )
                });

            let message = moves.map(|(from_name, number_of_tokens, targets)| {
                format!("{from_name} can move max {number_of_tokens} to: {:?}", targets.iter().map(|name| name.as_str()).collect::<Vec<&str>>().join(", "))
            }).collect::<Vec<String>>().join("\n");
            command.reply(format!("Moves: {}", message));
        }
    }
}