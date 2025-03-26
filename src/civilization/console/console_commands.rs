// use bevy::app::{App, Plugin};
// use bevy_console::{AddConsoleCommand, ConsoleConfiguration, ConsolePlugin};
// use crate::civilization::console::{build_city_console_command, expand_population_command, list_builds_command, list_moves_command, make_a_move_command, player_end_building_command, show_board_command, start_command, stupid_ai_command};
// use crate::civilization::console::build_city_console_command::BuildCityConsoleCommand;
// use crate::civilization::console::expand_population_command::ExpandPopulation;
// use crate::civilization::console::list_builds_command::ListBuildsCommand;
// use crate::civilization::console::list_moves_command::ListMoves;
// use crate::civilization::console::make_a_move_command::MakeAMove;
// use crate::civilization::console::player_end_building_command::PlayerEndBuildingCommand;
// use crate::civilization::console::show_board_command::ShowBoardCommand;
// use crate::civilization::console::start_command::StartCommand;
// use crate::civilization::console::stupid_ai_command::StupidAiCommand;
// pub struct CommandsPlugin;
// 
// /// This plugin handles player related stuff like movement
// /// Player logic is only active during the State `GameState::Playing`
// impl Plugin for CommandsPlugin {
//     fn build(&self, app: &mut App) {
//         app
//             .add_plugins(ConsolePlugin)
//             .insert_resource(ConsoleConfiguration {
//                 // override config here
//                 ..Default::default()
//             })
//             .add_console_command::<StartCommand, _>(start_command::start_command)
//             .add_console_command::<ExpandPopulation, _>(expand_population_command::expand_population)
//             .add_console_command::<ListMoves, _>(list_moves_command::list_moves)
//             .add_console_command::<MakeAMove, _>(make_a_move_command::make_a_move)
//             .add_console_command::<ShowBoardCommand, _>(show_board_command::show_board)
//             .add_console_command::<ListBuildsCommand, _>(list_builds_command::list_builds)
//             .add_console_command::<BuildCityConsoleCommand, _>(build_city_console_command::build_city)
//             .add_console_command::<PlayerEndBuildingCommand, _>(player_end_building_command::end_building)
//             .add_console_command::<StupidAiCommand, _>(stupid_ai_command::stupid_ai)
//         ;
//     }
// }
// 
// 
