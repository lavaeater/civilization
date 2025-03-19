use crate::civilization::concepts::trade_cards::enums::{Commodity, TradeCardType};
use bevy::prelude::{Entity, Event, Reflect};
use bevy::utils::HashMap;

#[derive(Event, Debug, Reflect)]
pub struct SendTradingCardsCommand {
    pub sending_player: Entity,
    pub receiving_player: Entity,
    pub cards_to_send: HashMap<TradeCardType, usize>
}

impl SendTradingCardsCommand {
    pub fn new(sending_player: Entity, receiving_player: Entity, cards_to_send: HashMap<TradeCardType, usize>) -> Self {
        SendTradingCardsCommand {
            sending_player,
            receiving_player,
            cards_to_send,
        }
    }
}