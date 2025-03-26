// use crate::civilization::components::game_moves_components::{AvailableMoves, Move};
// use bevy::core::Name;
// use bevy::prelude::Query;
// use bevy_console::ConsoleCommand;
// use clap::Parser;
// use itertools::Itertools;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "moves")]
// pub struct ListMoves;
// 
// pub fn list_moves(
//     mut command: ConsoleCommand<ListMoves>,
//     available_moves: Query<(&Name, &AvailableMoves)>,
//     name_query: Query<&Name>,
// ) {
//     if let Some(Ok(ListMoves {})) = command.take() {
//         for (name, avail_moves) in available_moves.iter() {
//             command.reply(format!("Player {} can perform the following moves", name));
// 
//             avail_moves.moves.iter().sorted_by(|(a,_),(b,_)|Ord::cmp(a,b)).for_each(|(index, game_move)| {
//                 match game_move {
//                     Move::PopulationExpansion(pop_exp_move) => {
//                         let area_name = name_query.get(pop_exp_move.area).unwrap();
//                         command.reply(format!("{} - Expand Population in {} with max {}", index, area_name, pop_exp_move.max_tokens));
//                     }
//                     Move::Movement(movement_move) => {
//                         let source_name = name_query.get(movement_move.source).unwrap();
//                         let target_name = name_query.get(movement_move.target).unwrap();
//                         command.reply(format!("{} - Move max {} tokens from {} to {}", index, movement_move.max_tokens, source_name, target_name));
//                     }
//                     Move::EndMovement => {
//                         command.reply(format!("{} - End Current Player Movement", index));
//                     }
//                     Move::CityConstruction(city_build_move) => {
//                         let target_name = name_query.get(city_build_move.target).unwrap();
//                         command.reply(format!("{} - Build city in {}", index, target_name));
//                     }
//                     Move::EndCityConstruction => {
//                         command.reply(format!("{} - End Current Player City Construction", index));
//                     }
//                     Move::EliminateCity(elimintation_move) => {
//                         let target_name = name_query.get(elimintation_move.area).unwrap();
//                         command.reply(format!("{} - Eliminate city at {}, gain {} tokens of the needed {}", index, target_name, elimintation_move.tokens_gained, elimintation_move.tokens_needed));
//                     }
//                     Move::AttackArea(movement_move) => {
//                         let source_name = name_query.get(movement_move.source).unwrap();
//                         let target_name = name_query.get(movement_move.target).unwrap();
//                         command.reply(format!("{} - Attack with max {} tokens from {} to {}", index, movement_move.max_tokens, source_name, target_name));
//                     }
//                     Move::AttackCity(movement_move) => {
//                         let source_name = name_query.get(movement_move.source).unwrap();
//                         let target_name = name_query.get(movement_move.target).unwrap();
//                         command.reply(format!("{} - Attack city with max {} tokens from {} to {}", index, movement_move.max_tokens, source_name, target_name));
//                     }
//                 }
//             });
//         }
//     }
// }
