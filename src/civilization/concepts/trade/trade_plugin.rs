use crate::civilization::concepts::trade::trade_events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::trade_resources::{TradeCountdown, TradeUiState};
use crate::civilization::concepts::trade::trade_triggers::{can_trade_removed, offer_published};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Plugin, Update};
use crate::civilization::concepts::trade::trade_systems::*;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TradeUiState::default()) // Placeholder until actual resources are added
            .init_resource::<TradeCountdown>()
            .add_message::<SendTradingCardsCommand>()
            .add_systems(OnEnter(GameActivity::Trade), setup_trade)
            .add_systems(
                Update,
                (
                    button_action,
                    trigger_trade_moves,
                    remove_rejected_trades,
                    delay_trade_moves_if_offers_are_accepted,
                    begin_trade_settlement,
                    handle_send_trading_cards_command,
                    settle_trades,
                    check_trade_gate,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(Update, button_action)
            .add_observer(offer_published)
            .add_observer(can_trade_removed);
    }
}
