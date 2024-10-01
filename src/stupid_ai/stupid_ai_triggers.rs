use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::stupid_ai::stupid_ai_components::StupidAi;
use crate::stupid_ai::stupid_ai_events::SelectStupidMove;
use bevy::log::debug;
use bevy::prelude::{EventWriter, OnAdd, Query, Trigger};

pub fn on_add_available_moves(
    trigger: Trigger<OnAdd, AvailableMoves>,
    is_stupid_ai: Query<&StupidAi>,
    mut event_writer: EventWriter<SelectStupidMove>,
) {
    debug!("on_add_available_moves");
    if is_stupid_ai.contains(trigger.entity()) {
        // debug!("Stupid AI detected");
        event_writer.send(SelectStupidMove::new(trigger.entity()));
    } else {
        debug!("Not a stupid AI");
    }
}