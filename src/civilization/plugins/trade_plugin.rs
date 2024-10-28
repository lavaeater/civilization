use crate::civilization::resources::prelude::TradeResources;
use crate::civilization::systems::trade_systems::ui_root;
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TradeResources>()
            .add_systems(OnEnter(GameActivity::Trade), (ui_root))
        ;
    }
}