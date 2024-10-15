use bevy::prelude::{App, Plugin};

pub struct TradeCardsPlugin;

impl Plugin for TradeCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(
            //     // OnEnter(GameActivity::AcquireTradeCards), find_trade_card_zones)
            //     // .add_systems(Update, (trade_card_gate).run_if(in_state(GameActivity::AcquireTradeCards)))
            //     // .observe(on_add_unresolved_trade_card)
            // )
        ;
    }
}