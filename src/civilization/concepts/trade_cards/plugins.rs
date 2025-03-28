use crate::civilization::concepts::trade_cards::components::CivilizationCardDefinitions;
use crate::civilization::concepts::trade_cards::events::{CheckIfWeCanTrade, HumanPlayerPulledTradeCard};
use crate::civilization::concepts::trade_cards::setup_systems::{load_civilization_cards, setup};
use crate::civilization::concepts::trade_cards::systems::{acquire_trade_cards, transition_to_trade};
use crate::{GameActivity, GameState};
use bevy::app::Startup;
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, Plugin, Update};
use bevy_common_assets::ron::RonAssetPlugin;

pub struct TradeCardPlugin;

impl Plugin for TradeCardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                RonAssetPlugin::<CivilizationCardDefinitions>::new(&["cards"]))
            .add_event::<CheckIfWeCanTrade>()
            .add_event::<HumanPlayerPulledTradeCard>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(GameState::Playing), load_civilization_cards)
            .add_systems(
                OnEnter(GameActivity::AcquireTradeCards), acquire_trade_cards)
            .add_systems(Update, transition_to_trade.run_if(in_state(GameActivity::AcquireTradeCards)))
        ;
    }
}

