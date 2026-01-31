use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::stupid_ai::stupid_ai_components::StupidAi;
use crate::stupid_ai::stupid_ai_events::SelectStupidMove;
use bevy::prelude::{MessageWriter, Add, Query, On};

pub fn on_add_available_moves(
    trigger: On<Add, AvailableMoves>,
    is_stupid_ai: Query<&StupidAi>,
    mut event_writer: MessageWriter<SelectStupidMove>,
) {
    if is_stupid_ai.contains(trigger.event().entity) {
        // //debug!("Stupid AI detected");
        event_writer.write(SelectStupidMove::new(trigger.event().entity));
    } else {
        //debug!("Not a stupid AI");
    }
}
