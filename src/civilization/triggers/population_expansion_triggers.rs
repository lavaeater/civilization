use crate::civilization::components::population_expansion_components::NeedsExpansion;
use crate::civilization::events::population_expansion_events::CheckGate;
use bevy::prelude::{EventWriter, OnRemove, Trigger};

pub fn on_remove_needs_expansion(
    _trigger: Trigger<OnRemove, NeedsExpansion>,
    mut event_writer: EventWriter<CheckGate>,
) {
    event_writer.send(CheckGate);
}