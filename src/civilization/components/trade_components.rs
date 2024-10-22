use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::enums::trade_card_enums::Commodity;

#[derive(Debug, Component, Reflect, Clone, Eq, PartialEq)]
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
    
    pub fn prepare_counter_offer(&self, new_initiator: Entity) -> TradeOffer {
        self.counter(new_initiator, None, None)
    }
    
    pub fn pay_more(&mut self, commodity: Commodity) {
        *self.initiator_commodities.entry(commodity).or_default() += 1;
    }
    
    pub fn pay_less(&mut self, commodity: Commodity) {
        if self.initiator_commodities.contains_key(&commodity) {
            let current_amount = self.initiator_commodities.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_commodities.remove(&commodity);
            }
        }
    }
    
    pub fn get_more(&mut self, commodity: Commodity) {
        *self.receiver_commodities.entry(commodity).or_default() += 1;
    }
    
    pub fn get_less(&mut self, commodity: Commodity) {
        if self.receiver_commodities.contains_key(&commodity) {
            let current_amount = self.receiver_commodities.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.receiver_commodities.remove(&commodity);
            }
        }
    }

    pub fn counter(
        &self,
        new_initiator: Entity,
        new_initiator_commodities: Option<HashMap<Commodity, usize>>,
        new_receiver_commodities: Option<HashMap<Commodity, usize>>,
    ) -> TradeOffer {
        // Create a new trade offer by cloning the current one
        let mut new_offer = self.clone();

        // Swap the initiator and receiver
        new_offer.receiver = Some(self.initiator);
        new_offer.initiator = new_initiator;

        //switch the commodities
        let temp = new_offer.initiator_commodities.clone();
        new_offer.initiator_commodities = new_offer.receiver_commodities.clone();
        new_offer.receiver_commodities = temp;
        
        // Update the commodities for the new initiator (if provided)
        if let Some(initiator_commodities) = new_initiator_commodities {
            new_offer.initiator_commodities = initiator_commodities;
        }

        // Update the commodities for the new receiver (if provided)
        if let Some(receiver_commodities) = new_receiver_commodities {
            new_offer.receiver_commodities = receiver_commodities;
        }

        // Clear the acceptances and rejections for the new offer
        new_offer.accepts.clear();
        new_offer.rejects.clear();

        new_offer
    }    
}

pub fn initiator_can_accept_trade_offer(offer: &TradeOffer, player_cards: &PlayerTradeCards) -> bool {
    // Step 1: Calculate the total number of cards required in the offer
    let total_required: usize = offer.initiator_required_cards();

    // Step 2: Check if the player has enough total tradeable cards
    if player_cards.number_of_tradeable_cards() < total_required {
        return false;
    }

    // Step 3: Check if the player has at least two cards of any commodity type in the offer
    let mut has_two_of_any = false;
    for (commodity, &required_quantity) in &offer.initiator_commodities {
        let player_quantity = player_cards.number_of_cards_of_commodity(commodity);
        if player_quantity >= 2 {
            has_two_of_any = true;
            break;
        } else if player_quantity >= 1 && required_quantity >= 1 {
            has_two_of_any = true;
            break;
        }
    }

    // Return true only if both conditions are satisfied
    has_two_of_any
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
        assert_eq!(initiator_can_accept_trade_offer(&trade_offer, &player_cards), true);
    }

    #[test]
    fn accept_trade_offer_test_fail() {
        let mut trade_offer = TradeOffer::new(create_entity());
        trade_offer.initiator_commodities.insert(Commodity::Salt, 2);
        let mut player_cards = PlayerTradeCards::default();
        player_cards.add_trade_card(TradeCard::new(1, TradeCardType::CommodityCard(Commodity::Ochre), true));
        player_cards.add_trade_card(TradeCard::new(5, TradeCardType::CommodityCard(Commodity::Wine), true));
        player_cards.add_trade_card(TradeCard::new(5, TradeCardType::CalamityCard(Calamity::BarbarianHordes), true));
        assert_eq!(initiator_can_accept_trade_offer(&trade_offer, &player_cards), false);
    }

    #[test]
    fn counter_trade_offer_test() {
        let initiator = create_entity();
        let receiver = create_entity();
        let mut trade_offer = TradeOffer::new(initiator);
        trade_offer.receiver = Some(receiver);
        trade_offer.initiator_commodities.insert(Commodity::Ochre, 2);
        trade_offer.receiver_commodities.insert(Commodity::Salt, 3);

        let new_initiator = create_entity();
        let new_initiator_commodities = Some(HashMap::from([(Commodity::Wine, 4)]));
        let new_receiver_commodities = Some(HashMap::from([(Commodity::Ochre, 1)]));

        let counter_offer = trade_offer.counter(new_initiator, new_initiator_commodities.clone(), new_receiver_commodities.clone());

        assert_eq!(counter_offer.initiator, new_initiator);
        assert_eq!(counter_offer.receiver, Some(initiator));
        assert_eq!(counter_offer.initiator_commodities, new_initiator_commodities.unwrap());
        assert_eq!(counter_offer.receiver_commodities, new_receiver_commodities.unwrap());
        assert!(counter_offer.accepts.is_empty());
        assert!(counter_offer.rejects.is_empty());
    }

    #[test]
    fn counter_trade_offer_test_no_commodities() {
        let initiator = create_entity();
        let receiver = create_entity();
        let mut trade_offer = TradeOffer::new(initiator);
        trade_offer.receiver = Some(receiver);
        trade_offer.initiator_commodities.insert(Commodity::Ochre, 2);
        trade_offer.receiver_commodities.insert(Commodity::Salt, 3);

        let new_initiator = create_entity();

        let counter_offer = trade_offer.counter(new_initiator, None, None);

        assert_eq!(counter_offer.initiator, new_initiator);
        assert_eq!(counter_offer.receiver, Some(initiator));
        assert_eq!(counter_offer.initiator_commodities, HashMap::from([(Commodity::Salt, 3)]));
        assert_eq!(counter_offer.receiver_commodities, HashMap::from([(Commodity::Ochre, 2)]));
        assert!(counter_offer.accepts.is_empty());
        assert!(counter_offer.rejects.is_empty());
    }
}

