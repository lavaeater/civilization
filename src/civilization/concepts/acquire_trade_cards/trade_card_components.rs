use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::{
    TradeCard, TradeCardTrait,
};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Color, Component, Entity, Reflect, ReflectComponent, Resource};
use itertools::Itertools;
use rand::RngExt;
use rand::seq::SliceRandom;

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
                .or_default()
                .extend(vec![trade_card; trade_card.number_of_cards()]);
        }
        // Shuffle each pile so calamities and commodities are mixed
        let mut rng = rand::rng();
        for pile in cards.values_mut() {
            pile.shuffle(&mut rng);
        }
        Self { card_piles: cards }
    }
    pub fn pull_card_from(&mut self, pile: usize) -> Option<TradeCard> {
        if let Some(p) = self.card_piles.get_mut(&pile) {
            p.pop()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerCardStack {
    pub card_type: TradeCard,
    pub count: usize,
    pub suite_value: usize,
    pub is_tradeable: bool,
    pub is_commodity: bool,
    pub is_calamity: bool,
    pub max_number_of_cards: usize,
    pub max_stack_value: usize,
}

#[derive(Component, Debug, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct PlayerTradeCards {
    cards: HashMap<TradeCard, usize>,
    /// Who most recently traded each held calamity card to this player.
    ///
    /// Rule 29.61 bars that player from being named a secondary victim of the
    /// calamity, and rule 30.221 makes them the beneficiary of Treachery. The
    /// provenance is only meaningful while the card is held, so it is dropped
    /// as soon as the card leaves the hand.
    calamity_traded_by: HashMap<TradeCard, Entity>,
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
            .copied()
            .collect_vec()
    }

    pub fn get_what_we_want(&self) -> Option<HashMap<TradeCard, usize>> {
        if self.can_trade()
            && let Some(top_commodity) = self.top_commodity()
        {
            return Some(HashMap::from([(top_commodity, 2)]));
        }
        None
    }

    pub fn get_what_we_can_pay(&self) -> Option<HashMap<TradeCard, usize>> {
        if self.can_trade()
            && let Some(bottom_commodity) = self.worst_commodity()
        {
            return Some(HashMap::from([(bottom_commodity, 2)]));
        }
        None
    }

    pub fn add_trade_card(&mut self, trade_card: TradeCard) {
        *self.cards.entry(trade_card).or_insert(0) += 1;
    }

    pub fn add_trade_cards(&mut self, trade_card: TradeCard, count: usize) {
        *self.cards.entry(trade_card).or_insert(0) += count;
    }

    /// Adds cards received in a trade, recording `from` as the trader for any
    /// calamity among them (rules 29.61/30.221).
    pub fn add_traded_cards(&mut self, trade_card: TradeCard, count: usize, from: Entity) {
        self.add_trade_cards(trade_card, count);
        if trade_card.is_calamity() {
            self.calamity_traded_by.insert(trade_card, from);
        }
    }

    /// Every recorded calamity provenance, for serialization.
    pub fn calamity_origins_as_vec(&self) -> Vec<(TradeCard, Entity)> {
        self.calamity_traded_by
            .iter()
            .map(|(card, from)| (*card, *from))
            .collect()
    }

    /// Restores a provenance entry (used when loading a save).
    pub fn set_calamity_traded_by(&mut self, trade_card: TradeCard, from: Entity) {
        self.calamity_traded_by.insert(trade_card, from);
    }

    /// Who traded this calamity to us, if anyone. `None` means it was drawn
    /// and kept -- rule 29.4's "retains a tradable calamity" case, where no
    /// other player benefits or is barred from secondary selection.
    pub fn calamity_traded_by(&self, trade_card: TradeCard) -> Option<Entity> {
        self.calamity_traded_by.get(&trade_card).copied()
    }

    pub fn has_trade_card(&self, trade_card: TradeCard) -> bool {
        self.cards.contains_key(&trade_card)
    }

    pub fn remove_worst_tradeable_calamity(&mut self) -> Option<TradeCard> {
        let calamity = self.worst_tradeable_calamity()?;
        self.remove_n_trade_cards(1, calamity).and(Some(calamity))
    }

    /// Removes and returns one physical card chosen uniformly at random from
    /// the whole hand (rule 24.51: an attacker who eliminates a city "draws,
    /// at random, one of the victim's trade cards"). Each individual card —
    /// not each distinct type — has an equal chance, so a 3-of-a-kind stack
    /// is three times as likely to be hit as a singleton. `None` if the hand
    /// is empty.
    pub fn remove_random_card(&mut self) -> Option<TradeCard> {
        let total = self.number_of_trade_cards();
        if total == 0 {
            return None;
        }
        let mut pick = rand::rng().random_range(0..total);
        for (card, count) in &self.cards {
            if pick < *count {
                let card = *card;
                self.remove_n_trade_cards(1, card);
                return Some(card);
            }
            pick -= count;
        }
        None
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

    /// The full hand with counts, sorted by card value — for serializing to
    /// the owning player (and only them: hands are hidden information).
    pub fn cards_with_counts(&self) -> Vec<(TradeCard, usize)> {
        let mut cards: Vec<(TradeCard, usize)> = self
            .cards
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(card, count)| (*card, *count))
            .collect();
        cards.sort_by_key(|(card, _)| (card.value(), format!("{card}")));
        cards
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
        top.zip(bottom)
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
        let commodity = self.worst_commodity()?;
        self.remove_n_trade_cards(1, commodity).and(Some(commodity))
    }

    pub fn worst_tradeable_calamity(&self) -> Option<TradeCard> {
        self.calamity_cards()
            .iter()
            .filter(|card| card.is_tradeable())
            .max_by(|a, b| a.value().cmp(&b.value()))
            .copied()
    }

    pub fn tradeable_calamity_cards_ranked(&self) -> Vec<TradeCard> {
        self.calamity_cards()
            .iter()
            .filter(|card| card.is_tradeable())
            .sorted_by_key(|c| c.value())
            .copied()
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

    /// Returns the total value of all commodity card stacks (count² × face_value for each)
    pub fn total_stack_value(&self) -> usize {
        self.commodity_card_suites().values().sum()
    }

    /// Mining card bonus (rule 28.53): the holder may increase the face value of
    /// one commodity card set by 1. The best set to boost is the one with the
    /// highest count, since the bonus = count² × 1.
    pub fn mining_bonus(&self) -> usize {
        self.commodity_cards()
            .values()
            .map(|&count| count * count)
            .max()
            .unwrap_or(0)
    }

    /// Total stack value, optionally including the Mining bonus.
    pub fn total_stack_value_with_mining(&self, has_mining: bool) -> usize {
        let base = self.total_stack_value();
        if has_mining {
            base + self.mining_bonus()
        } else {
            base
        }
    }

    /// Returns true if the player has any tradeable calamity cards
    pub fn has_tradeable_calamity(&self) -> bool {
        self.number_of_tradeable_calamity_cards() > 0
    }

    pub fn commodities(&self) -> HashSet<TradeCard> {
        self.cards
            .keys()
            .filter(|card| card.is_commodity())
            .copied()
            .collect()
    }

    pub fn trade_cards_grouped_by_value(&self) -> HashMap<usize, HashMap<TradeCard, usize>> {
        let mut grouped: HashMap<usize, HashMap<TradeCard, usize>> = HashMap::default();
        for (card, count) in &self.cards {
            let value = card.value();
            let entry = grouped.entry(value).or_default();
            *entry.entry(*card).or_insert(0) = *count;
        }
        grouped
    }

    pub fn as_card_stacks(&self) -> Vec<PlayerCardStack> {
        self.cards
            .iter()
            .map(|(card, &count)| {
                let max_cards = card.number_of_cards();
                PlayerCardStack {
                    card_type: *card,
                    count,
                    suite_value: if card.is_commodity() {
                        count * count * card.value()
                    } else {
                        0
                    },
                    is_tradeable: card.is_tradeable(),
                    is_commodity: card.is_commodity(),
                    is_calamity: card.is_calamity(),
                    max_number_of_cards: max_cards,
                    max_stack_value: if card.is_commodity() {
                        max_cards * max_cards * card.value()
                    } else {
                        0
                    },
                }
            })
            .collect()
    }

    pub fn as_card_stacks_sorted_by_value(&self) -> Vec<PlayerCardStack> {
        let mut stacks = self.as_card_stacks();
        stacks.sort_by_key(|b| std::cmp::Reverse(b.card_type.value()));
        stacks
    }

    pub fn remove_n_trade_cards(&mut self, n: usize, trade_card: TradeCard) -> Option<usize> {
        match self.cards.get_mut(&trade_card) {
            Some(count) => {
                if *count >= n {
                    *count -= n;
                    if *count == 0 {
                        self.cards.remove(&trade_card);
                        // The card is gone, and so is its provenance -- a
                        // calamity traded onwards must not keep pointing at
                        // whoever handed it to us.
                        self.calamity_traded_by.remove(&trade_card);
                    }
                    Some(n)
                } else {
                    None
                }
            }
            None => None, // No cards of this type
        }
    }

    /// Returns cards as a vector of (TradeCard, count) tuples for serialization
    pub fn cards_as_vec(&self) -> Vec<(TradeCard, usize)> {
        self.cards
            .iter()
            .map(|(card, count)| (*card, *count))
            .collect()
    }

    /// Restores cards from a vector of (TradeCard, count) tuples
    pub fn from_cards_vec(cards: Vec<(TradeCard, usize)>) -> Self {
        let mut player_cards = Self::default();
        for (card, count) in cards {
            player_cards.cards.insert(card, count);
        }
        player_cards
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_random_card_on_empty_hand_returns_none() {
        let mut hand = PlayerTradeCards::default();
        assert_eq!(hand.remove_random_card(), None);
    }

    /// Rule 24.51: draws exactly one card, and only one, from the hand.
    #[test]
    fn remove_random_card_removes_exactly_one_card() {
        let mut hand = PlayerTradeCards::default();
        hand.add_trade_card(TradeCard::Ochre);
        hand.add_trade_card(TradeCard::Salt);
        hand.add_trade_card(TradeCard::Iron);
        assert_eq!(hand.number_of_trade_cards(), 3);

        let drawn = hand.remove_random_card().expect("hand is non-empty");
        assert_eq!(hand.number_of_trade_cards(), 2);
        assert_eq!(hand.number_of_cards_for_trade_card(drawn), 0);
    }

    /// A single-card hand always yields that card, and leaves the hand empty.
    #[test]
    fn remove_random_card_single_card_hand_is_deterministic() {
        let mut hand = PlayerTradeCards::default();
        hand.add_trade_card(TradeCard::Iron);
        assert_eq!(hand.remove_random_card(), Some(TradeCard::Iron));
        assert_eq!(hand.number_of_trade_cards(), 0);
        assert_eq!(hand.remove_random_card(), None);
    }

    /// Every physical card gets picked over enough draws — regression guard
    /// against a bug that always removes from the same stack/type.
    #[test]
    fn remove_random_card_can_draw_each_distinct_type() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            let mut hand = PlayerTradeCards::default();
            hand.add_trade_card(TradeCard::Ochre);
            hand.add_trade_card(TradeCard::Salt);
            hand.add_trade_card(TradeCard::Iron);
            if let Some(card) = hand.remove_random_card() {
                seen.insert(card);
            }
        }
        assert_eq!(
            seen.len(),
            3,
            "expected all 3 card types to be drawn at least once across 200 trials"
        );
    }
}
