use bevy::prelude::{Entity, Resource, Timer, TimerMode, Reflect};
use bevy::platform::collections::HashMap;
use crate::civilization::concepts::acquire_trade_cards::TradeCard;

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
            trade_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
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
pub struct TradePhaseState {
    pub phase: TradePhase,
    pub countdown_seconds: f32,
    pub settling_trade: Option<Entity>,
    pub human_done: bool,
    pub create_offer_modal_open: bool,
    pub settlement_modal_open: bool,
    pub settling_offer_entity: Option<Entity>,
}

/// Tracks the offer being created in the Create Offer modal
#[derive(Resource, Default)]
pub struct CreateOfferState {
    /// Cards the player is offering (guaranteed - must be truthful)
    pub offering_guaranteed: HashMap<TradeCard, usize>,
    /// Number of hidden cards to offer
    pub offering_hidden_count: usize,
    /// Cards the player wants (guaranteed - what they request)
    pub wanting_guaranteed: HashMap<TradeCard, usize>,
    /// Number of hidden cards wanted
    pub wanting_hidden_count: usize,
}

impl CreateOfferState {
    pub fn reset(&mut self) {
        self.offering_guaranteed.clear();
        self.offering_hidden_count = 0;
        self.wanting_guaranteed.clear();
        self.wanting_hidden_count = 0;
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
    
    /// Check if offer is valid (min 3 cards each side, exactly 2 guaranteed each side)
    pub fn is_valid(&self) -> bool {
        self.guaranteed_offering_count() == 2
            && self.guaranteed_wanting_count() == 2
            && self.total_offering() >= 3
            && self.total_wanting() >= 3
    }
}
