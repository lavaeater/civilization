use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::{TradeCard, TradeCardTrait};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Color, Component, Reflect, Resource};
use itertools::Itertools;
use std::usize;

pub const MIN_CARDS_REQUIRED_TO_TRADE: usize = 5;

#[derive(Resource, Debug, Default)]
pub struct CivilizationTradeCards {
    pub card_piles: HashMap<usize, Vec<TradeCard>>,
}

impl CivilizationTradeCards {
    pub fn new() -> Self {
        let mut cards: HashMap<usize, Vec<TradeCard>> = HashMap::new();
        for trade_card in TradeCard::iter() {
            cards
                .entry(trade_card.value())
                .or_insert_with(Vec::new)
                .extend(vec![trade_card; trade_card.number_of_cards()]);
        }
        Self {
            card_piles: cards,
        }
    }
    pub fn pull_card_from(&mut self, pile: usize) -> Option<TradeCard> {
        if let Some(p) = self.card_piles.get_mut(&pile) {
            p.pop()
        } else {
            None
        }
    }
}

#[derive(Component, Debug, Reflect, Default, Clone)]
pub struct PlayerTradeCards {
    cards: HashMap<TradeCard, usize>,
}

impl PlayerTradeCards {
    pub fn can_trade(&self) -> bool {
        self.number_of_tradeable_cards() >= MIN_CARDS_REQUIRED_TO_TRADE // we have to trade away three cards to get one card... I say we need five to trade. For now!
    }

    pub fn wants(&self) -> Vec<TradeCard> {
        let mut commodities = self.commodity_card_suites().into_iter().collect::<Vec<_>>();
        commodities.sort_by_key(|(_commodity, value)| *value);
        commodities.reverse();
        commodities
            .iter()
            .take(3)
            .map(|(commodity, _)| commodity)
            .cloned()
            .collect_vec()
    }

    pub fn get_what_we_want(&self) -> Option<HashMap<TradeCard, usize>> {
        if self.can_trade() {
            if let Some(top_commodity) = self.top_commodity() {
                return Some(HashMap::from([(top_commodity, 2)]));
            }
        }
        None
    }

    pub fn get_what_we_can_pay(&self) -> Option<HashMap<TradeCard, usize>> {
        if self.can_trade() {
            if let Some(bottom_commodity) = self.worst_commodity() {
                return Some(HashMap::from([(bottom_commodity, 2)]));
            }
        }
        None
    }

    pub fn add_trade_card(&mut self, trade_card: TradeCard) {
        *self.cards.entry(trade_card).or_insert(0) += 1;
    }

    pub fn add_trade_cards(&mut self, trade_card: TradeCard, count: usize) {
        *self.cards.entry(trade_card).or_insert(0) += count;
    }

    pub fn has_trade_card(&self, trade_card: TradeCard) -> bool {
        self.cards.contains_key(&trade_card)
    }

    pub fn remove_worst_tradeable_calamity(&mut self) -> Option<TradeCard> {
        if let Some(calamity) = self.worst_tradeable_calamity() {
            match self.remove_n_trade_cards(1, calamity) {
                None => None,
                Some(_) => Some(calamity),
            }
        } else {
            None
        }
    }

    pub fn has_n_of_card(&self, n: usize, commodity: TradeCard) -> bool {
        self.number_of_cards_for_trade_card(commodity) >= n
    }

    pub fn number_of_cards_for_trade_card(&self, card: TradeCard) -> usize {
        *self.cards.get(&card).unwrap_or(&0)
    }

    pub fn has_trade_cards(&self) -> bool {
        !self.cards.is_empty()
    }

    pub fn number_of_trade_cards(&self) -> usize {
        self.cards.values().sum()
    }

    pub fn number_of_tradeable_cards(&self) -> usize {
        self.cards
            .iter()
            .filter(|(card, _)| card.is_tradeable())
            .map(|(_, count)| *count)
            .sum()
    }

    pub fn calamity_cards(&self) -> HashSet<TradeCard> {
        self.cards
            .iter()
            .filter(|(card, count)| card.is_calamity() && *count > &0)
            .map(|(card, _count)| *card)
            .collect()
    }

    pub fn commodity_cards(&self) -> HashMap<TradeCard, usize> {
        self.cards
            .iter()
            .filter(|(card, count)| card.is_commodity() && **count > 0)
            .map(|(card, count)| (*card, *count))
            .collect()
    }

    pub fn top_and_bottom_commodity(&self) -> Option<(TradeCard, TradeCard)> {
        let mut commodities = self.commodity_card_suites().into_iter().collect::<Vec<_>>();
        commodities.sort_by_key(|(_commodity, value)| *value);
        let top = commodities.last().map(|(commodity, _value)| *commodity);
        let bottom = commodities.first().map(|(commodity, _value)| *commodity);
        top.and_then(|top| bottom.map(|bottom| (top, bottom)))
    }

    pub fn top_commodity(&self) -> Option<TradeCard> {
        self.commodity_card_suites()
            .iter()
            .max_by_key(|(_commodity, value)| *value)
            .map(|(commodity, _value)| *commodity)
    }
    
    pub fn is_top_commodity(&self, card: TradeCard) -> bool {
        self.top_commodity() == Some(card)
    }

    pub fn worst_commodity(&self) -> Option<TradeCard> {
        self.commodity_card_suites()
            .iter()
            .min_by_key(|(_commodity, value)| *value)
            .map(|(commodity, _value)| *commodity)
    }

    pub fn remove_worst_commodity(&mut self) -> Option<TradeCard> {
        if let Some(commodity) = self.worst_commodity() {
            match self.remove_n_trade_cards(1, commodity) {
                None => None,
                Some(_) => Some(commodity),
            }
        } else {
            None
        }
    }

    pub fn worst_tradeable_calamity(&self) -> Option<TradeCard> {
        self
            .calamity_cards()
            .iter()
            .filter(|card| card.is_tradeable())
            .max_by(|a, b| a.value().cmp(&b.value()))
            .cloned()
    }

    pub fn tradeable_calamity_cards_ranked(&self) -> Vec<TradeCard> {
        self.calamity_cards()
            .iter()
            .filter(|card| card.is_tradeable())
            .sorted_by_key(|c| c.value())
            .cloned()
            .collect_vec()
    }

    pub fn number_of_tradeable_calamity_cards(&self) -> usize {
        self.calamity_cards()
            .iter()
            .filter(|card| card.is_tradeable())
            .count()
    }

    pub fn commodity_card_suites(&self) -> HashMap<TradeCard, usize> {
        self.commodity_cards()
            .iter()
            .map(|(card, count)| (*card, (*count * *count) * card.value()))
            .collect()
    }

    pub fn commodities(&self) -> HashSet<TradeCard> {
        self.cards
            .keys()
            .filter(|card| card.is_commodity())
            .cloned()
            .collect()
    }

    pub fn trade_cards_grouped_by_value(&self) -> HashMap<usize, HashMap<TradeCard, usize>> {
        let mut grouped: HashMap<usize, HashMap<TradeCard, usize>> = HashMap::default();
        for (card, count) in &self.cards {
            let value = card.value();
            let entry = grouped.entry(value).or_insert_with(HashMap::new);
            *entry.entry(*card).or_insert(0) = *count;
        }
        grouped
    }

    pub fn remove_n_trade_cards(&mut self, n: usize, trade_card: TradeCard) -> Option<usize> {
        match self.cards.get_mut(&trade_card) {
            Some(count) => {
                if *count >= n {
                    *count = *count - n;
                    if *count == 0 {
                        self.cards.remove(&trade_card);
                    }
                    Some(n)
                } else {
                    None
                }
            }
            None => None, // No cards of this type
        }
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
