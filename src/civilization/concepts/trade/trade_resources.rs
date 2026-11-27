use crate::civilization::concepts::acquire_trade_cards::TradeCard;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Entity, Reflect, Resource, Timer, TimerMode};

#[derive(Default, Resource)]
pub struct TradeUiState {
    pub human_player: Option<Entity>,
}

#[derive(Resource)]
pub struct TradeCountdown {
    pub trade_timer: Timer,
}

impl TradeCountdown {
    pub fn new() -> Self {
        Self {
            trade_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

impl Default for TradeCountdown {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum TradePhase {
    #[default]
    Trading,
    Settling,
    Ended,
}

#[derive(Resource, Default)]
// One flag per dialog and the human's done; an enum would not fit, they overlap.
#[allow(clippy::struct_excessive_bools)]
pub struct TradePhaseState {
    pub phase: TradePhase,
    pub countdown_seconds: f32,
    pub settling_trade: Option<Entity>,
    pub human_done: bool,
    pub create_offer_modal_open: bool,
    pub settlement_modal_open: bool,
    /// The "say something" dialog is up.
    pub talk_modal_open: bool,
    pub settling_offer_entity: Option<Entity>,
    /// Countdown when the last new offer went up: AIs stay at the table
    /// while offers keep coming.
    pub last_new_offer_at: f32,
    /// Real seconds per table second: `HUMAN_TABLE_PACE` while a human is
    /// still trading, 1 otherwise. 0 before the first frame counts as 1.
    pub pace: f32,
    /// Table seconds that passed this frame: what the AIs' clocks advance by.
    /// 0 while the clock is paused.
    pub table_delta: f32,
}

/// While a human is at the table the whole table runs this many times
/// slower: the countdown, and with it every AI's talking, proposing,
/// declining and leaving. A person needs a few seconds to read a line.
pub const HUMAN_TABLE_PACE: f32 = 3.0;
/// Table seconds an AI's fresh proposal to a human stays good (about half a
/// minute at the human pace): meanwhile its maker makes no other deal and
/// doesn't take it down, so the human has time to read it and accept.
pub const HUMAN_HOLD_SECONDS: f32 = 12.0;
/// Statements all AIs together may make per talk round with a human at the
/// table, so lines arrive one or two at a time instead of in bursts.
pub const HUMAN_TABLE_TALK_PER_ROUND: usize = 2;
/// Proposals all AIs together may post per round with a human at the table.
pub const HUMAN_TABLE_PROPOSALS_PER_ROUND: usize = 1;

impl TradePhaseState {
    pub fn pace(&self) -> f32 {
        self.pace.max(1.0)
    }

    /// How many statements (`talk`) or proposals the AIs may make this round.
    pub fn ai_budget(&self, talk: bool) -> usize {
        match (self.pace() > 1.0, talk) {
            (false, _) => usize::MAX,
            (true, true) => HUMAN_TABLE_TALK_PER_ROUND,
            (true, false) => HUMAN_TABLE_PROPOSALS_PER_ROUND,
        }
    }
}

/// Tracks the offer being created in the Create Offer modal
#[derive(Resource, Default)]
pub struct CreateOfferState {
    /// Cards the player is offering (guaranteed - must be truthful)
    pub offering_guaranteed: HashMap<TradeCard, usize>,
    /// Number of hidden cards to offer
    pub offering_hidden_count: usize,
    /// What the player says every hidden card is, if anything.
    pub offering_claim: Option<TradeCard>,
    /// Cards the player wants (guaranteed - what they request)
    pub wanting_guaranteed: HashMap<TradeCard, usize>,
    /// Number of hidden cards wanted
    pub wanting_hidden_count: usize,
    /// The player the proposal is made to; only they can accept it.
    pub target: Option<bevy::prelude::Entity>,
    /// The talk this proposal answers, if it was started from the feed.
    pub thread: Option<bevy::prelude::Entity>,
    /// Filled in from the feed: opening the dialog keeps it instead of
    /// starting blank.
    pub prefilled: bool,
}

impl CreateOfferState {
    pub fn reset(&mut self) {
        self.offering_guaranteed.clear();
        self.offering_hidden_count = 0;
        self.offering_claim = None;
        self.wanting_guaranteed.clear();
        self.wanting_hidden_count = 0;
        self.target = None;
        self.thread = None;
        self.prefilled = false;
    }

    /// Start from a draft made from the feed.
    pub fn prefill(&mut self, draft: crate::civilization::concepts::trade::ProposalDraft) {
        self.reset();
        self.offering_guaranteed = draft.offering;
        self.offering_hidden_count = draft.offering_hidden;
        self.wanting_guaranteed = draft.wanting;
        self.wanting_hidden_count = draft.wanting_hidden;
        self.target = draft.target;
        self.thread = draft.thread;
        self.prefilled = true;
    }

    /// The claim as stated on the offer: every hidden card is the
    /// chosen commodity.
    pub fn claimed(&self) -> HashMap<TradeCard, usize> {
        match self.offering_claim {
            Some(card) if self.offering_hidden_count > 0 => {
                HashMap::from_iter([(card, self.offering_hidden_count)])
            }
            _ => HashMap::default(),
        }
    }

    pub fn total_offering(&self) -> usize {
        self.offering_guaranteed.values().sum::<usize>() + self.offering_hidden_count
    }

    pub fn total_wanting(&self) -> usize {
        self.wanting_guaranteed.values().sum::<usize>() + self.wanting_hidden_count
    }

    pub fn guaranteed_offering_count(&self) -> usize {
        self.offering_guaranteed.values().sum()
    }

    pub fn guaranteed_wanting_count(&self) -> usize {
        self.wanting_guaranteed.values().sum()
    }

    /// Check if offer is valid: made to someone, min 3 cards each side, exactly
    /// 2 guaranteed each side.
    pub fn is_valid(&self) -> bool {
        self.target.is_some()
            && self.guaranteed_offering_count() == 2
            && self.guaranteed_wanting_count() == 2
            && self.total_offering() >= 3
            && self.total_wanting() >= 3
    }
}
