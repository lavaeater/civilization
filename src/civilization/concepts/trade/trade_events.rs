use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::TradeCard;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Entity, Message, Reflect};

#[derive(Message, Debug, Reflect)]
pub struct SendTradingCardsCommand {
    pub sending_player: Entity,
    pub receiving_player: Entity,
    pub cards_to_send: HashMap<TradeCard, usize>,
}

impl SendTradingCardsCommand {
    pub fn new(
        sending_player: Entity,
        receiving_player: Entity,
        cards_to_send: HashMap<TradeCard, usize>,
    ) -> Self {
        SendTradingCardsCommand {
            sending_player,
            receiving_player,
            cards_to_send,
        }
    }
}
