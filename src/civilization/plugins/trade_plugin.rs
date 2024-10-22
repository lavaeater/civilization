use crate::civilization::systems::prelude::setup_basic_trade_ui;
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameActivity::Trade), 
                         (setup_basic_trade_ui)
            )
        ;
    }
}