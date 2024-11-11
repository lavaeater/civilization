use crate::civilization::concepts::trade_cards::enums::Commodity::Ochre;
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

#[derive(Deserialize, Serialize, Asset, TypePath, Clone, Debug, Eq, PartialEq, Hash)]
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
        self.number_of_tradeable_cards() > 4 // we have to trade away three cards to get one card... I say we need five to trade. For now!
    }

    pub fn get_what_we_want(&self) -> Option<HashMap<Commodity, usize>> {
        if self.number_of_tradeable_cards() < 3 {
            return None;
        }
        if let Some((top_commodity, bottom_commodity)) = self.top_and_bottom_commodity() {
            return Some(HashMap::from([(top_commodity, 2), (bottom_commodity, 1)]));
        } else if let Some(top_commodity) = self.top_commodity() {
            return Some(HashMap::from([(top_commodity, 2), (Ochre, 1)]));
        }
        None
    }

    pub fn get_what_we_can_pay(&self) -> Option<HashMap<Commodity, usize>> {
        if self.number_of_tradeable_cards() < 3 {
            return None;
        }
        if let Some((top_commodity, bottom_commodity)) = self.top_and_bottom_commodity() {
            return Some(HashMap::from([(top_commodity, 2), (bottom_commodity, 1)]));
        } else if let Some(top_commodity) = self.top_commodity() {
            return Some(HashMap::from([(top_commodity, 2), (Ochre, 1)]));
        }
        None
    }

    pub fn add_trade_card(&mut self, trade_card: TradeCard) {
        self.trade_cards
            .entry(trade_card.card_type.clone())
            .or_default()
            .push(trade_card);
    }

    pub fn has_trade_card(&self, trade_card: &TradeCard) -> bool {
        self.trade_cards
            .get(&trade_card.card_type)
            .unwrap_or(&Vec::default())
            .contains(trade_card)
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
        self.trade_cards
            .values()
            .flatten()
            .filter(|card| card.tradeable)
            .count()
    }

    pub fn calamity_cards(&self) -> HashSet<TradeCard> {
        self.trade_cards
            .values()
            .flatten()
            .filter(|card| matches!(card.card_type, TradeCardType::CalamityCard(_)))
            .cloned()
            .collect()
    }

    pub fn commodity_cards(&self) -> HashSet<TradeCard> {
        self.trade_cards
            .values()
            .flatten()
            .filter(|card| matches!(card.card_type, TradeCardType::CommodityCard(_)))
            .cloned()
            .collect()
    }

    pub fn commodity_cards_by_commodity(&self) -> HashMap<Commodity, Vec<TradeCard>> {
        let mut cards: HashMap<Commodity, Vec<TradeCard>> = HashMap::default();
        for card in self.commodity_cards() {
            let commodity = card.get_commodity().unwrap();
            cards
                .entry(commodity)
                .or_insert_with(Vec::default)
                .push(card);
        }
        cards
    }

    pub fn top_and_bottom_commodity(&self) -> Option<(Commodity, Commodity)> {
        let mut commodities = self.commodity_card_suites().into_iter().collect::<Vec<_>>();
        commodities.sort_by_key(|(_commodity, value)| *value);
        let top = commodities.last().map(|(commodity, _value)| *commodity);
        let bottom = commodities.first().map(|(commodity, _value)| *commodity);
        top.and_then(|top| bottom.map(|bottom| (top, bottom)))
    }

    pub fn top_commodity(&self) -> Option<Commodity> {
        self.commodity_card_suites()
            .iter()
            .max_by_key(|(_commodity, value)| *value)
            .map(|(commodity, _value)| *commodity)
    }

    pub fn worst_commodity(&self) -> Option<Commodity> {
        self.commodity_card_suites()
            .iter()
            .min_by_key(|(_commodity, value)| *value)
            .map(|(commodity, _value)| *commodity)
    }

    pub fn commodity_card_suites(&self) -> HashMap<Commodity, usize> {
        let mut suits: HashMap<Commodity, usize> = HashMap::default();
        for (commodity, cards) in self.commodity_cards_by_commodity() {
            let card_value = cards.first().unwrap().value;
            *suits.entry(commodity).or_default() += (cards.len() ^ 2) * card_value;
        }
        suits
    }

    pub fn commodities(&self) -> HashSet<Commodity> {
        self.trade_cards
            .values()
            .flatten()
            .filter(|card| matches!(card.card_type, TradeCardType::CommodityCard(_)))
            .map(|c| c.get_commodity().unwrap())
            .unique()
            .collect()
    }

    pub fn cards_of_commodity_type(&self, commodity: &Commodity) -> Vec<TradeCard> {
        self.trade_cards
            .get(&TradeCardType::CommodityCard(*commodity))
            .unwrap_or(&Vec::default())
            .clone()
    }

    pub fn commodity_card_count(&self) -> HashMap<Commodity, usize> {
        let mut count: HashMap<Commodity, usize> = HashMap::default();
        for commodity in self.commodities() {
            count.insert(commodity, self.number_of_cards_of_commodity(&commodity));
        }
        count
    }

    pub fn trade_cards_grouped_by_value(&self) -> HashMap<usize, Vec<TradeCard>> {
        let mut grouped: HashMap<usize, Vec<TradeCard>> = HashMap::default();
        for (value, chunk) in &self.trade_cards().iter().chunk_by(|card| card.value) {
            grouped
                .entry(value)
                .or_insert_with(Vec::default)
                .extend(chunk.into_iter().cloned().collect::<Vec<_>>());
        }
        grouped
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Reflect)]
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
