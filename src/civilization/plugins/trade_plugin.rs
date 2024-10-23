use crate::civilization::systems::prelude::setup_human_trading_ui;
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};
use crate::civilization::resources::prelude::TradeOffers;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TradeOffers>()
            .add_systems(OnEnter(GameActivity::Trade),
                         setup_human_trading_ui,
            )
        ;
    }
}