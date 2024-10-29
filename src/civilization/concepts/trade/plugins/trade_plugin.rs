use crate::civilization::resources::prelude::TradeResources;
use crate::civilization::systems::prelude::{setup_human_trading_ui, setup_trade_ui};
use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use crate::GameActivity;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin)
            .insert_resource(TradeResources::default()) // Placeholder until actual resources are added
            .add_systems(OnEnter(GameActivity::Trade), setup_human_trading_ui)
        
        ;
    }
}