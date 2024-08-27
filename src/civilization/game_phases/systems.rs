use bevy::prelude::{EventReader, EventWriter, StateTransitionEvent};
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::game_phases::game_activity::GameActivity;

pub fn print_names_of_phases(
    mut write_line: EventWriter<PrintConsoleLine>,
    mut state_transition_event: EventReader<StateTransitionEvent<GameActivity>>
) {
    for event in state_transition_event.read() {
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Went from: {:?} to {:?}", event.exited, event.entered))));
    }
}