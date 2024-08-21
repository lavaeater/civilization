use bevy::app::{App, Plugin};
use bevy::prelude::EventWriter;
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsolePlugin, PrintConsoleLine};
use clap::Parser;

pub struct CommandsPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ConsolePlugin)
            .add_console_command::<LogCommand, _>(log_command);
    }
}

/// Example command
#[derive(Parser, ConsoleCommand)]
#[command(name = "log")]
struct LogCommand {
    msg: String,
}

fn log_command(mut command: ConsoleCommand<LogCommand>,
               mut console_line: EventWriter<PrintConsoleLine>) {
    if let Some(Ok(LogCommand { msg })) = command.take() {
        console_line.send(PrintConsoleLine::new(msg.into()));
    }
}