use crate::civilization::concepts::trade_cards::components::TradeCard;
use bevy::prelude::{Entity, Event, Reflect};
use uuid::Uuid;

#[derive(Event, Reflect, Default, Clone, PartialEq)]
pub struct CheckIfWeCanTrade;

#[derive(Event, Reflect, Clone, PartialEq)]
pub struct HumanPlayerPulledTradeCard {
    pub player_entity: Entity
}

impl HumanPlayerPulledTradeCard {
    pub fn new(player_entity: Entity) -> Self {
        HumanPlayerPulledTradeCard {
            player_entity
        }
    }
}