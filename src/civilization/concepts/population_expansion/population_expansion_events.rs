use bevy::prelude::{Entity, Message};

#[derive(Message, Debug)]
pub struct ExpandPopulationManuallyCommand {
    pub player: Entity,
    pub area: Entity,
    pub number_of_tokens: usize,
}

impl ExpandPopulationManuallyCommand {
    pub fn new(player: Entity, area: Entity, number_of_tokens: usize) -> Self {
        ExpandPopulationManuallyCommand {
            player,
            area,
            number_of_tokens,
        }
    }
}

#[derive(Message, Debug)]
pub struct CheckPlayerExpansionEligibility {
    pub player: Entity,
}

impl CheckPlayerExpansionEligibility {
    pub fn new(player: Entity) -> Self {
        CheckPlayerExpansionEligibility { player }
    }
}

#[derive(Message, Debug, Default)]
pub struct CheckGate;
