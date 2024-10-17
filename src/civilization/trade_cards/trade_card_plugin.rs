use crate::{GameActivity, GameState};
use bevy::app::Startup;
use bevy::prelude::{App, OnEnter, Plugin};
use bevy_common_assets::ron::RonAssetPlugin;
use crate::civilization::trade_cards::trade_card_components::CivilizationCardDefinitions;
use crate::civilization::trade_cards::trade_card_setup_systems::{load_civilization_cards, setup};
use crate::civilization::trade_cards::trade_card_systems::acquire_trade_cards;

pub struct TradeCardPlugin;

impl Plugin for TradeCardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                RonAssetPlugin::<CivilizationCardDefinitions>::new(&["cards"]))
            .add_systems(Startup, setup)
            .add_systems(OnEnter(GameState::Playing), load_civilization_cards)
            .add_systems(
                OnEnter(GameActivity::AcquireTradeCards), acquire_trade_cards)
        ;
    }
}

