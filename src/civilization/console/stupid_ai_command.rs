// use crate::player::Player;
// use crate::stupid_ai::stupid_ai_events::StupidAiEvent;
// use bevy::core::Name;
// use bevy::prelude::{Entity, EventWriter, Query, With};
// use bevy_console::ConsoleCommand;
// use clap::Parser;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "sa")]
// pub struct StupidAiCommand {
//     player: String,
// }
// 
// pub fn stupid_ai(
//     mut command: ConsoleCommand<StupidAiCommand>,
//     player_query: Query<(Entity, &Name), With<Player>>,
//     mut add_stupid_ai: EventWriter<StupidAiEvent>,
// ) {
//     if let Some(Ok(StupidAiCommand { player })) = command.take() {
//         if let Some(player_entity) = player_query.iter().find(|(_, name)| **name == Name::from(player.clone())).map(|(entity, _)| entity) {
//             add_stupid_ai.send(StupidAiEvent {
//                 player: player_entity
//             });
//             command.reply("Making Player Stupid");
//         } else {
//             command.reply("Could not find target player");
//         }
//     }
// }