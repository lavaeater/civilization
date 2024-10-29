use crate::civilization::concepts::trade::events::StartTrading;
use crate::civilization::concepts::trade::resources::TradeResources;
use crate::civilization::systems::prelude::{setup_human_trading_ui, setup_trade_ui};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin, Update};

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<StartTrading>()
            .insert_resource(TradeResources::default()) // Placeholder until actual resources are added
            .add_systems(OnEnter(GameActivity::Trade), setup_human_trading_ui)
            .add_systems(Update, setup_trade_ui.run_if(in_state(GameActivity::Trade)))
        ;
    }
}