use crate::civilization::concepts::trade::events::SendTradeCardsCommand;
use crate::civilization::concepts::trade::resources::{TradeCountdown, TradeUiState};
use crate::civilization::systems::prelude::{begin_trade_settlement, delay_trade_moves_if_offers_are_accepted, handle_trading_card_exchange, remove_rejected_trades, setup_trade, trade_ui, trigger_trade_moves};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin, Update};
use bevy_egui::EguiPlugin;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(TradeUiState::default()) // Placeholder until actual resources are added
            .init_resource::<TradeCountdown>()
            .add_event::<SendTradeCardsCommand>()
            .add_systems(OnEnter(GameActivity::Trade), setup_trade)
            .add_systems(
                Update,
                (
                    trade_ui,
                    trigger_trade_moves,
                    remove_rejected_trades, 
                    delay_trade_moves_if_offers_are_accepted,
                    begin_trade_settlement,
                    handle_trading_card_exchange
                )
                    .run_if(in_state(GameActivity::Trade)),
            );
    }
}
