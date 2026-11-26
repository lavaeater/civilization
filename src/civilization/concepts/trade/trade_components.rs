use crate::civilization::concepts::acquire_trade_cards::PlayerTradeCards;
use crate::civilization::concepts::acquire_trade_cards::TradeCard;
use crate::civilization::game_moves::TradeMove;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Component, Entity, Reflect};
use rand::prelude::{IndexedRandom, ThreadRng};
use std::collections::VecDeque;

#[derive(Component, Default, Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TradeButtonAction {
    #[default]
    Ok,
    Cancel,
    TradeAction(TradeMove),
}

#[derive(Component, Reflect, Debug)]
pub struct InSettlement;

#[derive(Component, Reflect, Debug, Default, Clone)]
pub struct PlayerSettlements {
    pub trades: VecDeque<Entity>,
    pub current_trade: Option<Entity>,
}

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

#[derive(Component, Default)]
pub struct PublishedOffersList;

#[derive(Debug, Reflect, Hash, Clone, Eq, PartialEq)]
pub enum AvailableTradeOfferActions {
    CanAccept,
    Decline,
}

#[derive(Component, Debug, Reflect, Hash, Clone, Eq, PartialEq)]
pub struct PlayerTradeInterests {
    pub wants: Vec<TradeCard>,
}

impl PlayerTradeInterests {
    pub fn from_trade_cards(player_trade_cards: &PlayerTradeCards) -> Self {
        Self {
            wants: player_trade_cards.wants(),
        }
    }

    pub fn get_trade_thingie(&self, rng: &mut ThreadRng) -> HashMap<TradeCard, usize> {
        HashMap::from([
            (*self.wants.choose(&mut *rng).unwrap(), 2),
            (*self.wants.choose(&mut *rng).unwrap(), 1),
        ])
    }
}
/*
Step one, make trade infinitely simpler than what we have right now.

Players only make tradesuggestions to each other and they either accept or decline, end
of story. No countering, no nothing. Everything else can grow from there.
 */

#[derive(Debug, Component, Reflect, Clone, Eq, PartialEq)]
pub struct TradeOffer {
    pub initiator: Entity,
    pub initiator_name: String,
    pub receiver: Entity,
    pub receiver_name: String,
    pub initiator_pays: HashMap<TradeCard, usize>,
    pub initiator_pays_guaranteed: HashMap<TradeCard, usize>, // The "true" cards that are guaranteed
    pub initiator_gets: HashMap<TradeCard, usize>,
    pub initiator_gets_guaranteed: HashMap<TradeCard, usize>, // The "true" cards that are guaranteed
    pub accepts: Option<Entity>,
    pub rejects: Option<Entity>,
    pub settled_players: HashSet<Entity>,
    pub initiator_cards: HashMap<TradeCard, usize>,
    pub receiver_cards: HashMap<TradeCard, usize>,
    pub system_settled: bool,
}

impl TradeOffer {
    pub fn am_i_involved_in_this_trade(&self, me: Entity) -> bool {
        self.am_i_the_initiator(me) || self.am_i_the_receiver(me)
    }

    pub fn am_i_the_initiator(&self, me: Entity) -> bool {
        self.initiator == me
    }

    pub fn am_i_the_receiver(&self, me: Entity) -> bool {
        self.receiver == me
    }

    pub fn get_trade_offer_actions(
        &self,
        entity: Entity,
    ) -> Option<HashSet<AvailableTradeOfferActions>> {
        let mut actions = HashSet::new();
        // One can only accept offers you are a part of
        if self.am_i_the_receiver(entity) {
            if self.can_be_accepted() {
                actions.insert(AvailableTradeOfferActions::CanAccept);
            } else {
                actions.insert(AvailableTradeOfferActions::Decline);
            }
        }

        if self.am_i_the_initiator(entity) && self.can_be_accepted() {
            actions.insert(AvailableTradeOfferActions::CanAccept);
        }
        if actions.is_empty() {
            None
        } else {
            Some(actions)
        }
    }

    pub fn propose_trade(
        initiator: Entity,
        initiator_name: impl Into<String>,
        receiver: Entity,
        receiver_name: impl Into<String>,
    ) -> Self {
        Self {
            initiator,
            initiator_name: initiator_name.into(),
            receiver,
            receiver_name: receiver_name.into(),
            initiator_pays: HashMap::default(),
            initiator_pays_guaranteed: HashMap::default(),
            initiator_gets: HashMap::default(),
            initiator_gets_guaranteed: HashMap::default(),
            accepts: None,
            rejects: None,
            settled_players: HashSet::default(),
            initiator_cards: HashMap::default(),
            receiver_cards: HashMap::default(),
            system_settled: false,
        }
    }

    pub fn can_be_accepted(&self) -> bool {
        self.gets_number_of_cards() > 2 && self.pays_number_of_cards() > 2 && self.rejects.is_none()
    }

    pub fn settle(&mut self, entity: Entity, cards_to_use: HashMap<TradeCard, usize>) -> bool {
        if self.am_i_involved_in_this_trade(entity) {
            if self.am_i_the_receiver(entity) {
                self.receiver_cards = cards_to_use;
            } else {
                self.initiator_cards = cards_to_use;
            }
            self.settled_players.insert(entity)
        } else {
            false
        }
    }

    pub fn is_player_settled(&self) -> bool {
        self.settled_players.contains(&self.receiver)
            && self.settled_players.contains(&self.initiator)
    }

    pub fn is_system_settled(&self) -> bool {
        self.system_settled
    }

    pub fn accept(&mut self, entity: &Entity) -> bool {
        if self.can_be_accepted() && self.am_i_the_receiver(*entity) {
            self.accepts = Some(*entity);
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, entity: Entity) {
        if self.am_i_the_receiver(entity) {
            self.rejects = Some(entity);
            self.accepts = None; // If you reject, you can't accept
        }
    }

    pub fn receiver_accepts(&self) -> bool {
        self.accepts == Some(self.receiver)
    }

    /// Returns true if both the initiator and the receiver have accepted the trade offer.
    /// This in turns means that there absolutely, 100% is a receiver.
    pub fn trade_accepted(&self) -> bool {
        self.receiver_accepts()
    }

    pub fn receiver_rejects(&self) -> bool {
        self.rejects.is_some()
    }

    pub fn trade_rejected(&self) -> bool {
        self.receiver_rejects()
    }

    pub fn pays_number_of_cards(&self) -> usize {
        self.initiator_pays.values().sum::<usize>()
            + self.initiator_pays_guaranteed.values().sum::<usize>()
    }

    pub fn gets_number_of_cards(&self) -> usize {
        self.initiator_gets.values().sum::<usize>()
            + self.initiator_gets_guaranteed.values().sum::<usize>()
    }

    pub fn guaranteed_pay_is_full(&self) -> bool {
        self.initiator_pays_guaranteed.values().sum::<usize>() == 2
    }

    pub fn guaranteed_get_is_full(&self) -> bool {
        self.initiator_gets_guaranteed.values().sum::<usize>() == 2
    }

    pub fn initiator_pays_more(&mut self, commodity: TradeCard) {
        if self.guaranteed_pay_is_full() {
            *self.initiator_pays.entry(commodity).or_insert(0) += 1;
        } else {
            *self.initiator_pays_guaranteed.entry(commodity).or_insert(0) += 1;
        }
    }

    pub fn pay_even_more(&mut self, commodity: TradeCard, count: usize) {
        for _ in 0..count {
            self.initiator_pays_more(commodity);
        }
    }

    pub fn pay_even_less(&mut self, commodity: TradeCard, count: usize) {
        for _ in 0..count {
            self.pay_less(commodity);
        }
    }

    pub fn pay_less(&mut self, commodity: TradeCard) {
        if self.initiator_pays.contains_key(&commodity) {
            let current_amount = self.initiator_pays.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_pays.remove(&commodity);
            }
        } else if self.initiator_pays_guaranteed.contains_key(&commodity) {
            let current_amount = self.initiator_pays_guaranteed.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_pays_guaranteed.remove(&commodity);
            }
        }
    }

    pub fn initiator_gets_more(&mut self, commodity: TradeCard) {
        if self.guaranteed_get_is_full() {
            *self.initiator_gets.entry(commodity).or_insert(0) += 1;
        } else {
            *self.initiator_gets_guaranteed.entry(commodity).or_insert(0) += 1;
        }
    }

    pub fn get_even_more(&mut self, commodity: TradeCard, count: usize) {
        for _ in 0..count {
            self.initiator_gets_more(commodity);
        }
    }

    pub fn get_even_less(&mut self, commodity: TradeCard, count: usize) {
        for _ in 0..count {
            self.get_less(commodity);
        }
    }

    pub fn get_less(&mut self, commodity: TradeCard) {
        if self.initiator_gets.contains_key(&commodity) {
            let current_amount = self.initiator_gets.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_gets.remove(&commodity);
            }
        } else if self.initiator_gets_guaranteed.contains_key(&commodity) {
            let current_amount = self.initiator_gets_guaranteed.get_mut(&commodity).unwrap();
            if *current_amount > 1 {
                *current_amount -= 1;
            } else {
                self.initiator_gets_guaranteed.remove(&commodity);
            }
        }
    }
}

/// " +n hidden", plus what was said about them if anything was.
pub fn hidden_text(hidden: usize, claimed: &HashMap<TradeCard, usize>) -> String {
    if hidden == 0 {
        return String::new();
    }
    if claimed.is_empty() {
        return format!(" +{hidden} hidden");
    }
    let mut said: Vec<(TradeCard, usize)> = claimed.iter().map(|(c, n)| (*c, *n)).collect();
    said.sort_by_key(|(c, _)| c.to_string());
    let said: Vec<String> = said.iter().map(|(c, n)| format!("{n}x {c}")).collect();
    format!(" +{hidden} hidden (said to be {})", said.join(", "))
}

/// A proposal at the trade table: a complete deal (rule 28.3) addressed to
/// one player, who alone may accept it (docs/trade-dialog-design.md).
#[derive(Debug, Component, Reflect, Clone, Eq, PartialEq)]
pub struct OpenTradeOffer {
    pub creator: Entity,
    pub creator_name: String,

    /// The player the proposal is made to; only they can accept it. A
    /// proposal without one cannot be accepted by anybody.
    pub target: Option<Entity>,
    pub target_name: Option<String>,
    /// The Want (`TradeTalk`) this proposal answers, if any.
    pub thread: Option<Entity>,

    /// What creator offers - guaranteed cards (exactly 2 total, must be truthful)
    pub offering_guaranteed: HashMap<TradeCard, usize>,
    /// How many additional hidden cards the creator offers
    pub offering_hidden_count: usize,
    /// What the creator *says* its hidden cards are. Talk, not a promise
    /// (rule 28.3: unnamed cards may be anything, "regardless of what was
    /// said"), so a bluffing AI can claim Salt and deliver a calamity.
    pub offering_claimed: HashMap<TradeCard, usize>,

    /// What creator wants - guaranteed cards (exactly 2 total, must be truthful)
    pub wanting_guaranteed: HashMap<TradeCard, usize>,
    /// How many additional hidden cards the creator wants
    pub wanting_hidden_count: usize,

    /// Who accepted this offer (if any)
    pub accepted_by: Option<Entity>,
    pub accepted_by_name: Option<String>,

    /// Whether the offer was rejected/withdrawn
    pub withdrawn: bool,

    /// Settlement: actual cards chosen by creator
    pub creator_actual_cards: Option<HashMap<TradeCard, usize>>,
    /// Settlement: actual cards chosen by acceptor
    pub acceptor_actual_cards: Option<HashMap<TradeCard, usize>>,
    /// Trade countdown when the offer went up, in tenths of a second.
    pub posted_at: u32,
}

impl OpenTradeOffer {
    pub fn new(
        creator: Entity,
        creator_name: impl Into<String>,
        target: Option<Entity>,
        target_name: Option<String>,
    ) -> Self {
        Self {
            creator,
            creator_name: creator_name.into(),
            target,
            target_name,
            thread: None,
            offering_guaranteed: HashMap::default(),
            offering_hidden_count: 0,
            offering_claimed: HashMap::default(),
            wanting_guaranteed: HashMap::default(),
            wanting_hidden_count: 0,
            accepted_by: None,
            accepted_by_name: None,
            withdrawn: false,
            creator_actual_cards: None,
            acceptor_actual_cards: None,
            posted_at: 0,
        }
    }

    /// Say what the hidden cards are (rule 28.3: talk, not a promise). Set
    /// the hidden count first: claims beyond it are dropped.
    pub fn claim_hidden(&mut self, claimed: impl IntoIterator<Item = (TradeCard, usize)>) {
        let mut left = self.offering_hidden_count;
        self.offering_claimed.clear();
        for (card, count) in claimed {
            let n = count.min(left);
            if n > 0 {
                *self.offering_claimed.entry(card).or_insert(0) += n;
                left -= n;
            }
        }
    }

    /// " +2 hidden (said to be 2x Salt)" for an offer list; empty when
    /// nothing is hidden.
    pub fn offering_hidden_text(&self) -> String {
        hidden_text(self.offering_hidden_count, &self.offering_claimed)
    }

    /// Total cards being offered
    pub fn total_offering(&self) -> usize {
        self.offering_guaranteed.values().sum::<usize>() + self.offering_hidden_count
    }

    /// Total cards being requested
    pub fn total_wanting(&self) -> usize {
        self.wanting_guaranteed.values().sum::<usize>() + self.wanting_hidden_count
    }

    /// Check if offer is valid (min 3 cards each side, exactly 2 guaranteed each side)
    pub fn is_valid(&self) -> bool {
        let guaranteed_offering: usize = self.offering_guaranteed.values().sum();
        let guaranteed_wanting: usize = self.wanting_guaranteed.values().sum();

        guaranteed_offering == 2
            && guaranteed_wanting == 2
            && self.total_offering() >= 3
            && self.total_wanting() >= 3
    }

    /// Can this player accept the offer?
    pub fn can_accept(&self, player: Entity) -> bool {
        // Can't accept your own offer
        if player == self.creator {
            return false;
        }
        // Already accepted or withdrawn
        if self.accepted_by.is_some() || self.withdrawn {
            return false;
        }
        // Only the player it is addressed to: nobody takes someone else's deal.
        self.target == Some(player)
    }

    /// Accept the offer
    pub fn accept(&mut self, player: Entity, player_name: impl Into<String>) -> bool {
        if self.can_accept(player) {
            self.accepted_by = Some(player);
            self.accepted_by_name = Some(player_name.into());
            true
        } else {
            false
        }
    }

    /// Is this offer in settlement phase?
    pub fn is_settling(&self) -> bool {
        self.accepted_by.is_some() && !self.is_settled()
    }

    /// Is this offer fully settled (both parties assigned cards)?
    pub fn is_settled(&self) -> bool {
        self.creator_actual_cards.is_some() && self.acceptor_actual_cards.is_some()
    }

    /// Settle creator's side
    pub fn settle_creator(&mut self, cards: HashMap<TradeCard, usize>) {
        self.creator_actual_cards = Some(cards);
    }

    /// Settle acceptor's side
    pub fn settle_acceptor(&mut self, cards: HashMap<TradeCard, usize>) {
        self.acceptor_actual_cards = Some(cards);
    }
}

/// Marker for the "Create Offer" modal
#[derive(Component, Default)]
pub struct CreateOfferModal;

/// Marker for the settlement modal
#[derive(Component, Default)]
pub struct SettlementModal;

/// Marker for the "Create Offer" button
#[derive(Component, Default)]
pub struct CreateOfferButton;

/// Marker for the "Done Trading" button
#[derive(Component, Default)]
pub struct DoneTradingButton;

#[cfg(test)]
mod claim_tests {
    use super::*;

    #[test]
    fn a_claim_never_covers_more_than_the_hidden_cards() {
        let mut offer = OpenTradeOffer::new(Entity::PLACEHOLDER, "A", None, None);
        offer.offering_hidden_count = 2;
        offer.claim_hidden([(TradeCard::Salt, 3), (TradeCard::Hides, 1)]);
        assert_eq!(offer.offering_claimed.get(&TradeCard::Salt), Some(&2));
        assert_eq!(offer.offering_claimed.get(&TradeCard::Hides), None);
        assert_eq!(
            offer.offering_hidden_text(),
            " +2 hidden (said to be 2x Salt)"
        );
    }

    #[test]
    fn no_claim_just_counts_hidden_cards() {
        let mut offer = OpenTradeOffer::new(Entity::PLACEHOLDER, "A", None, None);
        assert_eq!(offer.offering_hidden_text(), "");
        offer.offering_hidden_count = 1;
        assert_eq!(offer.offering_hidden_text(), " +1 hidden");
    }
}
