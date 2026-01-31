use bevy::prelude::{Entity, Message, Reflect};

#[derive(Message, Reflect, Default, Clone, PartialEq)]
pub struct CheckIfWeCanTrade;

#[derive(Message, Reflect, Clone, PartialEq)]
pub struct HumanPlayerTradeCardsUpdated {
    pub player_entity: Entity,
}

impl HumanPlayerTradeCardsUpdated {
    pub fn new(player_entity: Entity) -> Self {
        HumanPlayerTradeCardsUpdated { player_entity }
    }
}
