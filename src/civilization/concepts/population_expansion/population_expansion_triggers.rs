use crate::civilization::concepts::population_expansion::population_expansion_components::NeedsExpansion;
use crate::civilization::concepts::population_expansion::population_expansion_events::CheckGate;
use bevy::prelude::{MessageWriter, Remove, On};

pub fn on_remove_needs_expansion(
    _trigger: On<Remove, NeedsExpansion>,
    mut event_writer: MessageWriter<CheckGate>,
) {
    //debug!("NeedsExpansion component removed, sending CheckGate event");
    event_writer.write(CheckGate);
}
