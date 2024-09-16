use bevy::prelude::{Entity, Event};

#[derive(Event, Debug)]
pub struct CheckPlayerExpansionEligibility {
    pub player: Entity
}

impl CheckPlayerExpansionEligibility {
    pub fn new(player: Entity) -> Self {
        CheckPlayerExpansionEligibility { player }
    }
}