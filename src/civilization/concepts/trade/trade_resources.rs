use bevy::prelude::{Entity, Resource, Timer, TimerMode, Reflect};

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
}
