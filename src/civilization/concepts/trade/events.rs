use crate::civilization::concepts::trade_cards::enums::{Commodity, TradeCardType};
use bevy::prelude::{Entity, Event, Reflect};
use bevy::utils::HashMap;

#[derive(Event, Debug, Reflect)]
pub struct SendCardsToPlayerCommand {
    pub sending_player: Entity,
    pub target_player: Entity,
    pub cards: HashMap<TradeCardType, usize>,
}

impl SendCardsToPlayerCommand {
    pub fn new(source_player: Entity, target_player: Entity, cards: HashMap<TradeCardType, usize>) -> Self {
        SendCardsToPlayerCommand {
            sending_player: source_player,
            target_player,
            cards,
        }
    }
}