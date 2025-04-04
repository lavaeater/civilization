use bevy::prelude::{Entity, Resource, Timer, TimerMode};
#[derive(Default, Resource)]
pub struct TradeUiState {
    pub human_player: Option<Entity>,
    pub add_offered_commodity_open: bool,
    pub add_requested_commodity_open: bool,
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
