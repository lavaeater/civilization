use crate::civilization::concepts::trade_cards::enums::{Calamity, Commodity, TradeCardType};
use bevy::asset::Asset;
use bevy::prelude::{Component, Reflect, Resource, TypePath};
use bevy::utils::{HashMap, HashSet};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

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

#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerTradeCards {
    trade_cards: HashMap<TradeCardType, Vec<TradeCard>>,
}

impl PlayerTradeCards {
    pub fn can_trade(&self) -> bool {
        self.number_of_tradeable_cards() >= 3
    }
    
    pub fn add_trade_card(&mut self, trade_card: TradeCard) {
        self.trade_cards.entry(trade_card.card_type.clone()).or_insert_with(Vec::default).push(trade_card);
    }
    
    pub fn has_trade_card(&self, trade_card: &TradeCard) -> bool {
        self.trade_cards.get(&trade_card.card_type).unwrap_or(&Vec::default()).contains(trade_card)
    }
    
    pub fn has_n_commodities(&self, n: usize, commodity: &Commodity) -> bool {
        self.number_of_cards_of_commodity(commodity) >= n
    }
    
    pub fn number_of_cards_of_commodity(&self, commodity: &Commodity) -> usize {
        self.cards_of_commodity_type(commodity).len()
    }

    pub fn has_trade_cards(&self) -> bool {
        !self.trade_cards.is_empty()
    }
    
    pub fn trade_cards(&self) -> Vec<TradeCard> {
        self.trade_cards.values().flatten().cloned().collect()
    }

    pub fn number_of_trade_cards(&self) -> usize {
        self.trade_cards.values().map(|cards| cards.len()).sum()
    }
    
    pub fn number_of_tradeable_cards(&self) -> usize {
        self.trade_cards.values().flatten().filter(|card| card.tradeable).count()
    }

    pub fn calamity_cards(&self) -> HashSet<TradeCard> {
        self.trade_cards.values().flatten().filter(|card| matches!(card.card_type, TradeCardType::CalamityCard(_))).cloned().collect()
    }
    
    pub fn commodity_cards(&self) -> HashSet<TradeCard> {
        self.trade_cards.values().flatten().filter(|card| matches!(card.card_type, TradeCardType::CommodityCard(_))).cloned().collect()
    }

    pub fn commodities(&self) -> HashSet<Commodity> {
        self.trade_cards.values().flatten().filter(|card| matches!(card.card_type, TradeCardType::CommodityCard(_))).map(|c| c.get_commodity().unwrap()).unique().collect()
    }
    
    pub fn cards_of_commodity_type(&self, commodity: &Commodity) -> Vec<TradeCard> {
        self.trade_cards.get(&TradeCardType::CommodityCard(commodity.clone())).unwrap_or(&Vec::default()).clone()
    }
    
    pub fn trade_cards_grouped_by_value(&self) -> HashMap<usize, Vec<TradeCard>> {
        let mut grouped: HashMap<usize, Vec<TradeCard>> = HashMap::default();
        for (value, chunk) in &self.trade_cards().iter().chunk_by(|card| card.value) {
            grouped.entry(value).or_insert_with(Vec::default).extend(chunk.into_iter().cloned().collect::<Vec<_>>());
        }
        grouped
    }
}

#[derive(Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Reflect)]
pub struct TradeCard {
    pub value: usize,
    pub card_type: TradeCardType,
    pub tradeable: bool,
}

impl TradeCard {
    pub fn new(value: usize, card_type: TradeCardType, tradeable: bool) -> Self {
        Self {
            value,
            card_type,
            tradeable,
        }
    }
    
    pub fn is_commmodity(&self) -> bool {
        matches!(self.card_type, TradeCardType::CommodityCard(_))
    }
    
    pub fn is_calamity(&self) -> bool {
        matches!(self.card_type, TradeCardType::CalamityCard(_))
    }
    
    pub fn get_commodity(&self) -> Option<Commodity> {
        match &self.card_type {
            TradeCardType::CommodityCard(commodity) => Some(commodity.clone()),
            _ => None,
        }
    }
    
    pub fn get_calamity(&self) -> Option<Calamity> {
        match &self.card_type {
            TradeCardType::CalamityCard(calamity) => Some(calamity.clone()),
            _ => None,
        }
    }
}
