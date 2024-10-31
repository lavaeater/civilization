use crate::civilization::concepts::trade::resources::{TradeResources, TradeUiState};
use crate::civilization::systems::prelude::{setup_trade, trade_ui};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin, Update};
use bevy_egui::EguiPlugin;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin)
            .insert_resource(TradeResources::default()) // Placeholder until actual resources are added
            .insert_resource(TradeUiState::default()) // Placeholder until actual resources are added
            .add_systems(OnEnter(GameActivity::Trade), setup_trade)
            .add_systems(Update, trade_ui.run_if(in_state(GameActivity::Trade)))
        ;
    }
}