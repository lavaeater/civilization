use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Reflect, Default, Clone, PartialEq)]
pub struct CheckIfWeCanTrade;

#[derive(Event, Reflect, Clone, PartialEq)]
pub struct HumanPlayerTradeCardsUpdated {
    pub player_entity: Entity
}

impl HumanPlayerTradeCardsUpdated {
    pub fn new(player_entity: Entity) -> Self {
        HumanPlayerTradeCardsUpdated {
            player_entity
        }
    }
}