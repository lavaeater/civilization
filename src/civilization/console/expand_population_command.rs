// use bevy_console::ConsoleCommand;
// use bevy::prelude::{NextState, ResMut};
// use clap::Parser;
// use crate::GameActivity;
// 
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "popexp")]
// pub struct ExpandPopulation;
// 
// pub fn expand_population(
//     mut command: ConsoleCommand<ExpandPopulation>,
//     mut next_state: ResMut<NextState<GameActivity>>,
// ) {
//     if let Some(Ok(ExpandPopulation {})) = command.take() {
//         next_state.set(GameActivity::PopulationExpansion);
//         command.reply("We are starting the expansion!")
//     }
// }