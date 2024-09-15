use bevy::prelude::{EventWriter, OnAdd, Trigger};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::population_expansion::population_expansion_components::ExpandManually;

pub fn on_add_manual_expansion(
    trigger: Trigger<OnAdd, ExpandManually>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}