use bevy::app::{App, Plugin};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, PrintConsoleLine};
use clap::Parser;

pub struct CommandsPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ConsolePlugin)
            .insert_resource(ConsoleConfiguration {
                // override config here
                ..Default::default()
            })
            .add_console_command::<LogCommand, _>(log_command);
    }
}

/// Example command
#[derive(Parser, ConsoleCommand)]
#[command(name = "log")]
struct LogCommand {
    msg: String,
}

fn log_command(mut command: ConsoleCommand<LogCommand>) {
    if let Some(Ok(LogCommand { msg })) = command.take() {
    }
}