use crate::civilization::concepts::trade_cards::enums::Commodity;
use bevy::core::Name;
use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use std::collections::VecDeque;

#[derive(Component, Reflect, Debug)]
pub struct InSettlement;

#[derive(Component, Reflect, Debug, Default)]
pub struct PlayerSettlements {
    pub trades: VecDeque<Entity>,
    pub current_trade: Option<Entity>,
}

#[derive(Component, Reflect, Clone, Debug, PartialEq, Default)]
pub struct CanTrade;

#[derive(Component, Reflect, Clone, Debug, PartialEq, Default)]
pub struct NeedsTradeMove;

#[allow(dead_code)]
pub trait CoolDown {
    fn cool_down(&mut self, delta: f32) -> bool;
}

#[derive(Debug, Component, Reflect, Clone, Eq, PartialEq)]
pub struct PublishedOffer;

#[derive(Debug, Reflect, Hash, Clone, Eq, PartialEq)]
pub enum AvailableTradeOfferActions {
    Counter,
    Accept,
    Decline,
}

/*
Needs a bit of rethinking.
A trade offer consists of at least three cards. Of these, two **MUST** be true, and one **MUST** be
hidden but CAN be true. That means that whatever a user accepts, it has to be known that two are true, one is hidden.

We need to separate these things a bit,

So the user must actually specify what cards from his hand he will use when constructing a trade, and
we must then enable "masking" on the cards or something. 
*/

#[derive(Debug, Component, Reflect, Clone, Eq, PartialEq)]
pub struct TradeOffer {
    pub initiator: Entity,
    pub initiator_name: Name,
    pub receiver: Option<Entity>,
    pub receiver_name: Option<Name>,
    pub initiator_pays: HashMap<Commodity, usize>,
    pub initiator_gets: HashMap<Commodity, usize>,
    pub accepts: HashSet<Entity>,
    pub rejects: Option<Entity>,
}

impl TradeOffer {
    pub fn new(initiator: Entity, initiator_name: Name) -> Self {
        TradeOffer {
            initiator,
            initiator_name,
            receiver: None,
            receiver_name: None,
            initiator_pays: HashMap::default(),
            initiator_gets: HashMap::default(),
            accepts: HashSet::default(),
            rejects: None,
        }
    }

    pub fn get_trade_offer_actions(&self, entity: Entity) -> Option<HashSet<AvailableTradeOfferActions>> {
        let mut actions = HashSet::new();

        // One can only accept offers you are a part of
        if self.receiver == Some(entity) {
            if self.can_be_accepted() {
                actions.insert(AvailableTradeOfferActions::Accept);
            }
            actions.insert(AvailableTradeOfferActions::Decline);
        }

        if self.initiator == entity {
            if self.can_be_accepted() {
                actions.insert(AvailableTradeOfferActions::Accept);
            }
            actions.insert(AvailableTradeOfferActions::Decline);
        } else {
            //We can counter all offers we haven't created ourselves
            actions.insert(AvailableTradeOfferActions::Counter);
        }

        if actions.is_empty() {
            None
        } else {
            Some(actions)
        }
    }

    pub fn is_open_offer(&self, entity: Entity) -> bool {
        self.receiver.is_none() && self.receiver_name.is_none() && self.initiator != entity
    }

    pub fn propose_trade_to(
        initiator: Entity,
        initiator_name: Name,
        receiver: Entity,
        receiver_name: Name,
    ) -> Self {
        TradeOffer {
            initiator,
            initiator_name,
            receiver: Some(receiver),
            receiver_name: Some(receiver_name),
            initiator_pays: HashMap::default(),
            initiator_gets: HashMap::default(),
            accepts: HashSet::default(),
            rejects: None,
        }
    }

    pub fn create_open_offer(
        initiator: Entity,
        initiator_name: Name,
        initiator_gets: HashMap<Commodity, usize>,
    ) -> Self {
        TradeOffer {
            initiator,
            initiator_name,
            receiver: None,
            receiver_name: None,
            initiator_pays: HashMap::default(),
            initiator_gets,
            accepts: HashSet::default(),
            rejects: None,
        }
    }

    pub fn initiator_accepts(&self) -> bool {
        self.accepts.contains(&self.initiator)
    }

    pub fn can_be_accepted(&self) -> bool {
        self.receiver.is_some()
            && self.receiver_name.is_some()
            && self.gets_number_of_cards() > 2
            && self.pays_number_of_cards() > 2
    }

    pub fn needs_counter(&self) -> bool {
        self.pays_number_of_cards() < 3
    }

    pub fn accept(&mut self, entity: Entity) -> bool {
        if self.can_be_accepted() && (self.receiver == Some(entity) || self.initiator == entity) {
            self.accepts.insert(entity);
            return true;
        }
        false
    }

    pub fn reject(&mut self, entity: Entity) {
        if self.receiver == Some(entity) {
            self.rejects = Some(entity);
        }
    }

    pub fn receiver_accepts(&self) -> bool {
        match self.receiver {
            Some(entity) => self.accepts.contains(&entity),
            None => false,
        }
    }

    /// Returns true if both the initiator and the receiver have accepted the trade offer.
    /// This in turns means that there absolutely, 100% is a receiver.
    pub fn trade_accepted(&self) -> bool {
        self.initiator_accepts() && self.receiver_accepts()
    }

    pub fn receiver_rejects(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn trade_rejected(&self) -> bool {
        self.receiver_rejects()
    }

    pub fn pays_number_of_cards(&self) -> usize {
        self.initiator_pays.values().sum()
    }

    pub fn gets_number_of_cards(&self) -> usize {
        self.initiator_gets.values().sum()
    }

    pub fn prepare_counter_offer(&self, new_initiator: Entity) -> TradeOffer {
        self.counter(new_initiator, None, None)
    }

    pub fn pay_more(&mut self, commodity: Commodity) {
        *self.initiator_pays.entry(commodity).or_default() += 1;
    }

    pub fn pay_less(&mut self, commodity: Commodity) {
        if self.initiator_pays.contains_key(&commodity) {
            let current_amount = self.initiator_pays.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_pays.remove(&commodity);
            }
        }
    }

    pub fn get_more(&mut self, commodity: Commodity) {
        *self.initiator_gets.entry(commodity).or_default() += 1;
    }

    pub fn get_less(&mut self, commodity: Commodity) {
        if self.initiator_gets.contains_key(&commodity) {
            let current_amount = self.initiator_gets.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_gets.remove(&commodity);
            }
        }
    }

    pub fn counter(
        &self,
        new_initiator: Entity,
        new_pays: Option<HashMap<Commodity, usize>>,
        new_gets: Option<HashMap<Commodity, usize>>,
    ) -> TradeOffer {
        // Create a new trade offer by cloning the current one
        let mut new_offer = self.clone();

        // Swap the initiator and receiver
        new_offer.receiver = Some(self.initiator);
        new_offer.initiator = new_initiator;

        //switch the commodities
        let temp = new_offer.initiator_pays.clone();
        new_offer.initiator_pays = new_offer.initiator_gets.clone();
        new_offer.initiator_gets = temp;

        // Update the commodities for the new initiator (if provided)
        if let Some(commodities) = new_pays {
            new_offer.initiator_pays = commodities;
        }

        // Update the commodities for the new receiver (if provided)
        if let Some(commodities) = new_gets {
            new_offer.initiator_gets = commodities;
        }

        // Clear the acceptances and rejections for the new offer
        new_offer.accepts.clear();
        new_offer.rejects = None;

        new_offer
    }
}
