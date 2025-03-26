// use clap::Parser;
// use bevy_console::ConsoleCommand;
// use bevy::prelude::{Commands, Entity, Query};
// use bevy::core::Name;
// use crate::civilization::components::city_construction_components::IsBuilding;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "eb")]
// pub struct PlayerEndBuildingCommand {
//     pub player_name: String,
// }
// 
// pub fn end_building(
//     mut command: ConsoleCommand<PlayerEndBuildingCommand>,
//     mut commands: Commands,
//     player_query: Query<(Entity, &Name)>,
// ) {
//     if let Some(Ok(PlayerEndBuildingCommand { player_name })) = command.take() {
//         if let Some(player_entity) = player_query
//             .iter()
//             .find(|(_, name)| **name == Name::from(player_name.clone()))
//             .map(|(entity, _)| entity) {
//             commands.entity(player_entity).insert(IsBuilding {});
//         }
//     }
// }