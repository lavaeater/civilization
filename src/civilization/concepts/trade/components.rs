use crate::civilization::concepts::trade_cards::enums::Commodity;
use bevy::core::Name;
use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};

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

#[derive(Debug, Component, Reflect, Clone, Eq, PartialEq)]
pub struct TradeOffer {
    pub initiator: Entity,
    pub initiator_name: Name,
    pub receiver: Option<Entity>,
    pub receiver_name: Option<Name>,
    pub initiator_pays: HashMap<Commodity, usize>,
    pub initiator_receives: HashMap<Commodity, usize>,
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
            initiator_receives: HashMap::default(),
            accepts: HashSet::default(),
            rejects: None,
        }
    }
    
    pub fn propose_trade_to(initiator: Entity, initiator_name: Name, receiver: Entity, receiver_name: Name) -> Self {
        TradeOffer {
            initiator,
            initiator_name,
            receiver: Some(receiver),
            receiver_name: Some(receiver_name),
            initiator_pays: HashMap::default(),
            initiator_receives: HashMap::default(),
            accepts: HashSet::default(),
            rejects: None,
        }
    }

    pub fn create_open_offer(initiator: Entity,
                             initiator_name: Name,
                             initiator_wants: HashMap<Commodity, usize>) -> Self {
        TradeOffer {
            initiator,
            initiator_name,
            receiver: None,
            receiver_name: None,
            initiator_pays: HashMap::default(),
            initiator_receives: initiator_wants,
            accepts: HashSet::default(),
            rejects: None,
        }
    }

    pub fn initiator_accepts(&self) -> bool {
        self.accepts.contains(&self.initiator)
    }
    
    pub fn can_be_accepted(&self) -> bool {
        self.receiver.is_some() && self.receiver_name.is_some() && self.receives_number_of_cards() > 2 && self.pays_number_of_cards() > 2
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

    pub fn receives_number_of_cards(&self) -> usize {
        self.initiator_receives.values().sum()
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
        *self.initiator_receives.entry(commodity).or_default() += 1;
    }

    pub fn get_less(&mut self, commodity: Commodity) {
        if self.initiator_receives.contains_key(&commodity) {
            let current_amount = self.initiator_receives.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_receives.remove(&commodity);
            }
        }
    }

    pub fn counter(
        &self,
        new_initiator: Entity,
        new_payment: Option<HashMap<Commodity, usize>>,
        new_recives: Option<HashMap<Commodity, usize>>,
    ) -> TradeOffer {
        // Create a new trade offer by cloning the current one
        let mut new_offer = self.clone();

        // Swap the initiator and receiver
        new_offer.receiver = Some(self.initiator);
        new_offer.initiator = new_initiator;

        //switch the commodities
        let temp = new_offer.initiator_pays.clone();
        new_offer.initiator_pays = new_offer.initiator_receives.clone();
        new_offer.initiator_receives = temp;

        // Update the commodities for the new initiator (if provided)
        if let Some(initiator_commodities) = new_payment {
            new_offer.initiator_pays = initiator_commodities;
        }

        // Update the commodities for the new receiver (if provided)
        if let Some(receiver_commodities) = new_recives {
            new_offer.initiator_receives = receiver_commodities;
        }

        // Clear the acceptances and rejections for the new offer
        new_offer.accepts.clear();
        new_offer.rejects = None;

        new_offer
    }
}