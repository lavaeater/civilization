use crate::civilization::systems::prelude::setup_human_trading_ui;
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};
use crate::civilization::concepts::trade::resources::TradeResources;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TradeResources::default()) // Placeholder until actual resources are added
            .add_systems(OnEnter(GameActivity::Trade), setup_human_trading_ui)
        
        ;
    }
}