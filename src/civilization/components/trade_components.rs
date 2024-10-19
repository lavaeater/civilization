use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::enums::trade_card_enums::Commodity;

#[derive(Debug, Component, Reflect, Default)]
pub struct TradeOffer {
    pub player_a: Option<Entity>,
    pub player_b: Option<Entity>,
    pub player_a_commodities: HashMap<Commodity, usize>,
    pub player_b_commodities: HashMap<Commodity, usize>,
    pub accepts: HashSet<Entity>,
    pub rejects: HashSet<Entity>,
}

impl TradeOffer {
    pub fn new() -> Self {
        TradeOffer {
            player_a: None,
            player_b: None,
            player_a_commodities: HashMap::default(),
            player_b_commodities: HashMap::default(),
            accepts: HashSet::default(),
            rejects: HashSet::default(),
        }
    }
    
    pub fn player_a_accepts(&self) -> bool {
        match self.player_a {
            Some(entity) => self.accepts.contains(&entity),
            None => false,
        }
    }
    
    pub fn player_b_accepts(&self) -> bool {
        match self.player_b {
            Some(entity) => self.accepts.contains(&entity),
            None => false,
        }
    }
    
    pub fn trade_accepted(&self) -> bool {
        self.player_a_accepts() && self.player_b_accepts()
    }
    
    pub fn player_a_rejects(&self) -> bool {
        match self.player_a {
            Some(entity) => self.rejects.contains(&entity),
            None => false,
        }
    }
    
    pub fn player_b_rejects(&self) -> bool {
        match self.player_b {
            Some(entity) => self.rejects.contains(&entity),
            None => false,
        }
    }
    
    pub fn trade_rejected(&self) -> bool {
        self.player_a_rejects() || self.player_b_rejects()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;
    use std::cell::RefCell;

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
    
    
}

