// use crate::civilization::components::*;
// use crate::civilization::events::MoveTokensFromStockToAreaCommand;
// use crate::player::Player;
// use crate::GameActivity;
// use bevy::core::Name;
// use bevy::prelude::{Entity, MessageWriter, NextState, Query, ResMut, With};
// use bevy_console::ConsoleCommand;
// use clap::Parser;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "start")]
// pub struct StartCommand;
// 
// pub fn start_command(
//     mut command: ConsoleCommand<StartCommand>,
//     player_query: Query<(Entity, &Name, &Faction), With<Player>>,
//     start_area_query: Query<(Entity, &Name, &StartArea)>,
//     mut writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
//     mut next_state: ResMut<NextState<GameActivity>>,
// ) {
//     if let Some(Ok(StartCommand {})) = command.take() {
//         for (player_entity, name, player_faction) in player_query.iter() {
//             if let Some((area_entity, area_name, _)) = start_area_query.iter().find(|(_, _, start_area)| start_area.faction == player_faction.faction) {
//                 writer.write(
//                     MoveTokensFromStockToAreaCommand {
//                         area_entity,
//                         player_entity,
//                         number_of_tokens: 1,
//                     });
//                 command.reply(format!("{:#?} adds a token to {:#?}!", name, area_name));
//             }
//         }
//         next_state.set(GameActivity::PopulationExpansion);
//     }
// }