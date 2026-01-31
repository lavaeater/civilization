// use clap::Parser;
// use bevy_console::ConsoleCommand;
// use bevy::prelude::Query;
// use bevy::core::Name;
// use crate::civilization::components::city_construction_components::CityBuildTargets;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "builds")]
// pub struct ListBuildsCommand;
// 
// pub fn list_builds(
//     mut command: ConsoleCommand<ListBuildsCommand>,
//     player_query: Query<(&Name, &CityBuildTargets)>,
//     name_query: Query<&Name>,
// ) {
//     if let Some(Ok(ListBuildsCommand {})) = command.take() {
//         for (player_name, targets) in player_query.iter() {
//             let target_names = targets.targets.iter().map(|target| name_query.get(*target).unwrap().as_str()).collect::<Vec<&str>>().join(", ");
//             command.reply(format!("{:#?} can build in: {:#?}", player_name, target_names));
//         }
//     }
// }