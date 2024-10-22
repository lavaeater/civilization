use crate::civilization::components::prelude::TradeOffer;
use bevy::prelude::{Reflect, Resource};
use std::collections::VecDeque;

#[derive(Resource, Debug, Reflect, Default)]
pub struct TradeOffers {
    pub offers: VecDeque<TradeOffer>,
}

impl TradeOffers {
    pub fn add_offer(&mut self, offer: TradeOffer) {
        self.offers.push_back(offer);
    }

    pub fn remove_offer(&mut self, offer: &TradeOffer) {
        self.offers.retain(|x| x != offer);
    }
}