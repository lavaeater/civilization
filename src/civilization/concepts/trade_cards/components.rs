use crate::civilization::concepts::trade_cards::enums::Commodity::Ochre;
use crate::civilization::concepts::trade_cards::enums::{Calamity, Commodity, TradeCardType};
use bevy::asset::Asset;
use bevy::prelude::{Color, Component, Reflect, Resource, TypePath};
use bevy::utils::{HashMap, HashSet};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub color: String
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

#[derive(Component, Debug, Reflect, Default, Clone)]
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
        let mut commodities = self.commodity_card_suites().into_iter().collect::<Vec<_>>();
        commodities.sort_by_key(|(_commodity, value)| *value);
        Some(commodities[0..2].iter().cloned().collect())
    }

    pub fn add_trade_card(&mut self, trade_card: TradeCard) {
        self.trade_cards
            .entry(trade_card.card_type)
            .or_default()
            .push(trade_card);
    }

    pub fn add_trade_cards(&mut self, trade_cards: Vec<TradeCard>) {
        for card in trade_cards.iter() {
            self.add_trade_card(card.clone());
        }
    }

    pub fn has_trade_card(&self, trade_card: &TradeCard) -> bool {
        self.trade_cards
            .get(&trade_card.card_type)
            .unwrap_or(&Vec::default())
            .contains(trade_card)
    }
    
    pub fn remove_worst_tradeable_calamity(&mut self) -> Option<TradeCard> {
        if let Some(calamity) = self.worst_tradeable_calamity() {
            self.remove_card_for_calamity(&calamity)
        } else { None }
    }
    
    pub fn remove_card_for_calamity(&mut self, calamity: &Calamity) -> Option<TradeCard>{
        if let Some(cards) = self.trade_cards
            .get_mut(&TradeCardType::CalamityCard(*calamity)) {
            let card = cards.pop();
            if cards.is_empty() {
                self.trade_cards.remove(&TradeCardType::CalamityCard(*calamity));
            }
            card
        } else { None }
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

    pub fn worst_tradeable_calamity(&self) -> Option<Calamity> {
        match self.calamity_cards()
            .iter()
            .filter(|card| card.tradeable)
            .max_by(|a,b| a.value.cmp(&b.value)) {
            None => { None }
            Some(card) => { card.get_calamity()}
        }
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

    pub fn trade_cards_grouped_by_value_and_type(&self) -> HashMap<usize, HashMap<TradeCardType, Vec<TradeCard>>> {
        let mut grouped: HashMap<usize, HashMap<TradeCardType, Vec<TradeCard>>> = HashMap::default();
        for (value, chunk) in &self.trade_cards().iter().chunk_by(|card| card.value) {
            let mut by_type: HashMap<TradeCardType, Vec<TradeCard>> = HashMap::default();
            for card in chunk {
                by_type
                    .entry(card.card_type)
                    .or_insert_with(Vec::default)
                    .push(card.clone());
            }
            grouped.insert(value, by_type);
        }
        grouped
    }

    pub fn remove_n_trade_cards(&mut self, n: usize, trade_card_type: TradeCardType) -> Option<Vec<TradeCard>> {
        match self.trade_cards.get_mut(&trade_card_type) {
            Some(vector) => {
                if vector.len() >= n {
                    let removed_cards: Vec<TradeCard> = vector.drain(vector.len() - n..).collect();
                    if vector.is_empty() {
                        self.trade_cards.remove(&trade_card_type);
                    }
                    Some(removed_cards)
                } else {
                    None // Not enough cards of this type
                }
            },
            None => None // No cards of this type
        }
    }
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct TradeCard {
    pub id: Uuid,
    pub value: usize,
    pub card_type: TradeCardType,
    pub tradeable: bool,
    pub color: Color
}

impl Eq for TradeCard {}

impl std::hash::Hash for TradeCard {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.value.hash(state);
        self.card_type.hash(state);
        self.tradeable.hash(state);
        // Skip hashing the Color field as it doesn't implement Hash
        // Instead, we can hash some of its components if needed
        (self.color.to_srgba().red as u32).hash(state);
        (self.color.to_srgba().green as u32).hash(state);
        (self.color.to_srgba().blue as u32).hash(state);
        (self.color.to_srgba().alpha as u32).hash(state);
    }
}
pub fn hex(hex_str: &str) -> Color {
    let hex_str = hex_str.trim_start_matches('#');
    
    if hex_str.len() != 6 && hex_str.len() != 8 {
        return Color::WHITE;
    }
    
    let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap_or(255);
    
    let a = if hex_str.len() == 8 {
        u8::from_str_radix(&hex_str[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    
    Color::srgba_u8(r, g, b, a)
}

impl TradeCard {
    pub fn new(value: usize, card_type: TradeCardType, tradeable: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            card_type,
            tradeable,
            color: Color::WHITE
        }
    }
    
    pub fn from_def(def: &TradeCardDefinition) -> Self {
        Self {
            id: Uuid::new_v4(),
            value: def.value,
            card_type: def.card_type,
            tradeable: def.tradeable,
            color: hex(def.color.as_str())
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
