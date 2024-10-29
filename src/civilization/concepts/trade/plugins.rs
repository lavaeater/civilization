use crate::civilization::concepts::trade::resources::TradeResources;
use crate::civilization::systems::prelude::trade_ui;
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, Plugin, Update};
use bevy_egui::EguiPlugin;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin)
            .insert_resource(TradeResources::default()) // Placeholder until actual resources are added
            .add_systems(Update, trade_ui.run_if(in_state(GameActivity::Trade)))
        ;
    }
}