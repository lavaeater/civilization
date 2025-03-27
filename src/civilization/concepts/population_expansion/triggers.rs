use crate::civilization::concepts::population_expansion::components::NeedsExpansion;
use crate::civilization::concepts::population_expansion::events::CheckGate;
use bevy::prelude::{EventWriter, OnRemove, Trigger};

pub fn on_remove_needs_expansion(
    _trigger: Trigger<OnRemove, NeedsExpansion>,
    mut event_writer: EventWriter<CheckGate>,
) {
    //debug!("NeedsExpansion component removed, sending CheckGate event");
    event_writer.send(CheckGate);
}