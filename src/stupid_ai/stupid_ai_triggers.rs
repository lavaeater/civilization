use crate::civilization::AvailableMoves;
use crate::civilization::DebugOptions;
use crate::stupid_ai::{AiMoveQueue, SelectStupidMove, StupidAi};
use bevy::prelude::*;

pub fn on_add_available_moves(
    trigger: On<Add, AvailableMoves>,
    is_stupid_ai: Query<&StupidAi>,
    mut queue: ResMut<AiMoveQueue>,
    debug_options: Res<DebugOptions>,
) {
    if is_stupid_ai.contains(trigger.event().entity) {
        queue.push(trigger.event().entity, debug_options.ai_move_delay_secs);
    }
}

pub fn drain_ai_move_queue(
    time: Res<Time>,
    mut queue: ResMut<AiMoveQueue>,
    mut event_writer: MessageWriter<SelectStupidMove>,
) {
    if queue.pending.is_empty() {
        return;
    }

    let dt = time.delta_secs();
    let mut i = 0;
    while i < queue.pending.len() {
        queue.pending[i].1 -= dt;
        if queue.pending[i].1 <= 0.0 {
            let (player, _) = queue.pending.remove(i);
            event_writer.write(SelectStupidMove::new(player));
        } else {
            i += 1;
        }
    }
}
