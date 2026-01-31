// use crate::civilization::components::population::Population;
// use crate::civilization::components::*;
// use bevy::core::Name;
// use bevy::prelude::{Has, Query};
// use bevy_console::ConsoleCommand;
// use clap::Parser;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "board")]
// pub struct ShowBoardCommand;
// 
// pub fn show_board(
//     mut command: ConsoleCommand<ShowBoardCommand>,
//     area_query: Query<(&Name, &Population, Has<StartArea>, Has<CitySite>, Has<BuiltCity>, Has<Volcano>, Has<FloodPlain>)>,
//     player_areas_query: Query<(&Name, &PlayerAreas, &TokenStock)>,
//     name_query: Query<&Name>,
// ) {
//     if let Some(Ok(ShowBoardCommand {})) = command.take() {
//         for (area_name, population, is_start_area, is_city_site, has_city, _has_volcano, _has_flood_plain) in area_query.iter().filter(|(_, population, _, _, _, _, _)| population.has_population()) {
//             command.reply(format!("Area: {} {} has population: {}{}{}", area_name, if is_start_area { "<start>" } else { "" }, population.total_population(), if is_city_site { ", City Site" } else { "" }, if has_city { ", Has City" } else { "" }));
//         }
// 
//         for (player_name, player_areas, stock) in player_areas_query.iter() {
//             command.reply(format!("Player: {} has {} population in the following areas:", player_name, player_areas.total_population()));
//             for area in player_areas.areas_with_population() {
//                 command.reply(format!("  - {} : {}", name_query.get(area).unwrap(), player_areas.population_in_area(area)));
//             }
//             command.reply(format!("Player: {} has {} tokens in stock", player_name, stock.tokens_in_stock()));
//         }
//     }
// }