// use crate::civilization::components::game_moves_components::{AvailableMoves, Move};
// use crate::civilization::events::city_construction_events::{BuildCityCommand, EndPlayerCityConstruction};
// use crate::civilization::events::movement_events::{MoveTokenFromAreaToAreaCommand, PlayerMovementEnded};
// use bevy::core::Name;
// use bevy::prelude::{Entity, MessageWriter, Query};
// use bevy_console::ConsoleCommand;
// use clap::Parser;
// use crate::civilization::concepts::population_expansion::events::ExpandPopulationManuallyCommand;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "move")]
// pub struct MakeAMove {
//     pub player: String,
//     pub index: usize,
//     pub number: Option<usize>,
// }
// 
// pub fn make_a_move(
//     mut command: ConsoleCommand<MakeAMove>,
//     available_moves: Query<(&Name, &AvailableMoves, Entity)>,
//     mut expand_writer: MessageWriter<ExpandPopulationManuallyCommand>,
//     mut move_tokens_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>,
//     mut end_movement_writer: MessageWriter<PlayerMovementEnded>,
//     mut build_city_writer: MessageWriter<BuildCityCommand>,
//     mut end_player_city_construction: MessageWriter<EndPlayerCityConstruction>
// ) {
//     if let Some(Ok(MakeAMove { player, index, number })) = command.take() {
//         for (name, avail_moves, player_entity) in available_moves.iter() {
//             if name.to_string() == player {
//                 avail_moves.moves.iter().for_each(|(move_index, game_move)| {
//                     if index.eq(move_index) {
//                         match game_move {
//                             Move::PopulationExpansion(pop_exp_move) => {
//                                 expand_writer.write(ExpandPopulationManuallyCommand::new(player_entity, pop_exp_move.area, number.unwrap_or(pop_exp_move.max_tokens)));
//                             }
//                             Move::Movement(movement_move) => { 
//                                 move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, number.unwrap_or(movement_move.max_tokens), player_entity));
//                             }
//                             Move::EndMovement => {
//                                 end_movement_writer.write(PlayerMovementEnded::new(player_entity));
//                             }
//                             Move::CityConstruction(build_city_move) => {
//                                 build_city_writer.write(BuildCityCommand::new(player_entity, build_city_move.target));
//                             }
//                             Move::EndCityConstruction => {
//                                 end_player_city_construction.write(EndPlayerCityConstruction::new(player_entity));
//                             }
//                             Move::EliminateCity(_el_move) => {
//                                 
//                             }
//                             Move::AttackArea(movement_move) => {
//                                 move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, number.unwrap_or(movement_move.max_tokens), player_entity));
//                             }
//                             Move::AttackCity(movement_move) => {
//                                 move_tokens_writer.write(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, number.unwrap_or(movement_move.max_tokens), player_entity));
//                             }
//                         }
//                     }
//                 });
//             }
//         }
//     }
// }
