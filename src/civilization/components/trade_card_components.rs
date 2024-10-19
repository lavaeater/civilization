use bevy::prelude::{Component, Reflect, Resource, TypePath};
use bevy::utils::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use bevy::asset::Asset;
use itertools::Itertools;

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

#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerTradeCards {
    trade_cards: Vec<TradeCard>,
}

impl PlayerTradeCards {
    pub fn add_trade_card(&mut self, trade_card: TradeCard) {
        self.trade_cards.push(trade_card);
    }

    pub fn trade_cards(&self) -> Vec<TradeCard> {
        self.trade_cards.clone()
    }

    pub fn has_trade_card(&self, trade_card: &TradeCard) -> bool {
        self.trade_cards.contains(trade_card)
    }

    pub fn has_trade_cards(&self) -> bool {
        !self.trade_cards.is_empty()
    }

    pub fn number_of_trade_cards(&self) -> usize {
        self.trade_cards.len()
    }

    pub fn calamity_cards(&self) -> HashSet<TradeCard> {
        self.trade_cards.iter().filter(|card| matches!(card.card_type, TradeCardType::CalamityCard(_))).cloned().collect()
    }
    
    pub fn commodity_cards(&self) -> HashSet<TradeCard> {
        self.trade_cards.iter().filter(|card| matches!(card.card_type, TradeCardType::CommodityCard(_))).cloned().collect()
    }
    
    pub fn cards_of_commodity_type(&self, commodity: Commodity) -> Vec<TradeCard> {
        self.trade_cards.iter().filter(|card| {
            if let TradeCardType::CommodityCard(c) = &card.card_type {
                c == &commodity
            } else {
                false
            }
        }).cloned().collect()
    }

    pub fn trade_cards_grouped_by_value(&self) -> HashMap<usize, Vec<TradeCard>> {
        let mut grouped: HashMap<usize, Vec<TradeCard>> = HashMap::default();
        for (value, chunk) in &self.trade_cards.iter().chunk_by(|card| card.value) {
            grouped.entry(value).or_insert_with(Vec::default).extend(chunk.into_iter().cloned().collect::<Vec<_>>());
        }
        grouped
    }
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum TradeCardType {
    CommodityCard(Commodity),
    CalamityCard(Calamity),
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum Commodity {
    Ochre,
    Hides,
    Iron,
    Papyrus,
    Salt,
    Timber,
    Grain,
    Oil,
    Cloth,
    Wine,
    Bronze,
    Silver,
    Spices,
    Resin,
    Gems,
    Dye,
    Gold,
    Ivory,
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum Calamity {
    VolcanoEarthquake,
    Treachery,
    Famine,
    Superstition,
    CivilWar,
    SlaveRevolt,
    Flood,
    BarbarianHordes,
    Epidemic,
    CivilDisorder,
    IconoclasmAndHeresy,
    Piracy,
}

impl TradeCard {
    pub fn new(value: usize, card_type: TradeCardType, tradeable: bool) -> Self {
        Self {
            value,
            card_type,
            tradeable,
        }
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