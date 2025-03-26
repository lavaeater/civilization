use bevy::prelude::{debug, EventWriter, OnRemove, Trigger};
use crate::civilization::concepts::population_expansion::components::NeedsExpansion;
use crate::civilization::concepts::population_expansion::events::CheckGate;

pub fn on_remove_needs_expansion(
    _trigger: Trigger<OnRemove, NeedsExpansion>,
    mut event_writer: EventWriter<CheckGate>,
) {
    debug!("NeedsExpansion component removed, sending CheckGate event");
    event_writer.send(CheckGate);
}