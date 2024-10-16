use bevy::prelude::{Component, Reflect, Resource, TypePath};
use bevy::utils::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use bevy::asset::Asset;
use crate::civilization::general::prelude::*;

#[derive(Component, Debug, Reflect, Default)]
pub struct TradeInterests {
    pub commodities: HashSet<Commodity>,
}

#[derive(Deserialize, Serialize, Asset, TypePath, Clone)]
pub struct CivilizationCardDefinitions {
    pub trading_cards: Vec<TradeCardDefinition>,
}

#[derive(
    Deserialize,
    Serialize,
    Asset,
    TypePath,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
)]
pub struct TradeCardDefinition {
    pub value: usize,
    pub card_type: TradeCardType,
    pub tradeable: bool,
    pub number: usize,
}

#[derive(Resource, Debug, Default)]
pub struct CivilizationTradeCards {
    pub card_piles: HashMap<usize, Vec<TradeCard>>,
}

impl CivilizationTradeCards {
    pub fn pull_card_from(&mut self, pile: usize) -> Option<TradeCard> {
        self.card_piles.get_mut(&pile).unwrap().pop()
    }
}