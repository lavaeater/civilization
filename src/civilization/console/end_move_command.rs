use bevy_console::ConsoleCommand;
use bevy::prelude::EventWriter;
use clap::Parser;
use crate::civilization::movement::movement_events::ClearAllMoves;

#[derive(Parser, ConsoleCommand)]
#[command(name = "endmove")]
pub struct EndMoveCommand;

pub fn end_move(
    mut command: ConsoleCommand<EndMoveCommand>,
    mut clear_all_moves: EventWriter<ClearAllMoves>,
) {
    if let Some(Ok(EndMoveCommand {})) = command.take() {
        clear_all_moves.send(ClearAllMoves {});
        command.reply("Next player started!");
    }
}