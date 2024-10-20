use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::enums::trade_card_enums::Commodity;

#[derive(Debug, Component, Reflect)]
pub struct TradeOffer {
    pub initiator: Entity,
    pub receiver: Option<Entity>,
    pub initiator_commodities: HashMap<Commodity, usize>,
    pub receiver_commodities: HashMap<Commodity, usize>,
    pub accepts: HashSet<Entity>,
    pub rejects: HashSet<Entity>,
}

impl TradeOffer {
    pub fn new(initiator: Entity) -> Self {
        TradeOffer {
            initiator,
            receiver: None,
            initiator_commodities: HashMap::default(),
            receiver_commodities: HashMap::default(),
            accepts: HashSet::default(),
            rejects: HashSet::default(),
        }
    }

    pub fn initiator_accepts(&self) -> bool {
        self.accepts.contains(&self.initiator)
    }

    pub fn receiver_accepts(&self) -> bool {
        match self.receiver {
            Some(entity) => self.accepts.contains(&entity),
            None => false,
        }
    }

    pub fn trade_accepted(&self) -> bool {
        self.initiator_accepts() && self.receiver_accepts()
    }

    pub fn initiator_rejects(&self) -> bool {
        self.rejects.contains(&self.initiator)
    }

    pub fn receiver_rejects(&self) -> bool {
        match self.receiver {
            Some(entity) => self.rejects.contains(&entity),
            None => false,
        }
    }

    pub fn trade_rejected(&self) -> bool {
        self.initiator_rejects() || self.receiver_rejects()
    }

    pub fn initiator_required_cards(&self) -> usize {
        self.initiator_commodities.values().sum()
    }

}


pub fn initiator_can_accept_trade_offer(offer: &TradeOffer, player_cards: PlayerTradeCards) -> bool {
    // All trades require at least 3 cards to be traded.
    // A trade can contain more cards, so if that is the case the initiator must have that amount
    if player_cards.number_of_tradeable_cards() < offer.initiator_required_cards() || player_cards.number_of_tradeable_cards() < 3 {
        return false;
    }

    if offer.initiator_commodities.iter().all(|(commodity, amount)| {
        player_cards.has_n_commodities(*amount, commodity)
    }) {
        true
    } else {
        //example: trade with 4 cards, 2 ochre, 2 salt. But only two ever need to be true.
        let mut count = 0;
        for (commodity, amount) in &offer.initiator_commodities {
            
            if player_cards.has_n_commodities(1, commodity) {
                count += 1;
            }
            if count >= 2 {
                return true;
            }
        }
        false
    }
}

pub fn receiver_can_accept_trade_offer(offer: &TradeOffer, player_cards: PlayerTradeCards) -> bool {
    offer.initiator_commodities.iter().all(|(commodity, amount)| {
        player_cards.has_n_commodities(*amount, commodity)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;
    use std::cell::RefCell;
    use crate::civilization::components::prelude::TradeCard;
    use crate::civilization::enums::prelude::{Calamity, TradeCardType};

    thread_local! {
    static ENTITY_COUNTER: RefCell<u32> = RefCell::new(0);
}
    fn create_entity() -> Entity {
        ENTITY_COUNTER.with(|counter| {
            let index = *counter.borrow();
            *counter.borrow_mut() += 1; // Increment the counter for the next entity
            Entity::from_raw(index)
        })
    }
    
    #[test]
    fn accept_trade_offer_test() {
        let mut trade_offer = TradeOffer::new(create_entity());
        trade_offer.initiator_commodities.insert(Commodity::Ochre, 2);
        trade_offer.initiator_commodities.insert(Commodity::Salt, 2);
        let mut player_cards = PlayerTradeCards::default();
        player_cards.add_trade_card(TradeCard::new(1, TradeCardType::CommodityCard(Commodity::Ochre), true));
        player_cards.add_trade_card(TradeCard::new(1, TradeCardType::CommodityCard(Commodity::Ochre), true));
        player_cards.add_trade_card(TradeCard::new(5, TradeCardType::CommodityCard(Commodity::Wine), true));
        player_cards.add_trade_card(TradeCard::new(5, TradeCardType::CalamityCard(Calamity::BarbarianHordes), true));
        assert_eq!(initiator_can_accept_trade_offer(&trade_offer, player_cards), true);
    }
}

