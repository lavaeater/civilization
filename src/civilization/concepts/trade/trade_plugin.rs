use crate::civilization::concepts::trade::trade_events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::trade_resources::{CreateOfferState, TradeCountdown, TradePhaseState, TradeUiState};
use crate::civilization::concepts::trade::trade_triggers::{can_trade_removed, offer_published};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update};
use crate::civilization::concepts::trade::trade_systems::*;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TradeUiState::default())
            .init_resource::<TradeCountdown>()
            .init_resource::<TradePhaseState>()
            .init_resource::<CreateOfferState>()
            .add_message::<SendTradingCardsCommand>()
            .add_systems(OnEnter(GameActivity::Trade), (setup_trade, setup_trade_phase_ui))
            .add_systems(OnExit(GameActivity::Trade), cleanup_trade_phase_ui)
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
                    update_trade_countdown_display,
                    update_player_trading_status,
                    update_open_offers_display,
                    handle_done_trading_button,
                    handle_create_offer_button,
                    handle_accept_offer_button,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(
                Update,
                (
                    spawn_create_offer_modal,
                    handle_close_create_offer_modal,
                    despawn_create_offer_modal,
                    handle_offer_card_selection,
                    handle_want_card_selection,
                    handle_hidden_count_buttons,
                    update_hidden_count_displays,
                    update_offer_summary_display,
                    handle_publish_offer,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(
                Update,
                (
                    check_for_settlement_needed,
                    spawn_settlement_modal,
                    handle_settlement_card_selection,
                    update_settlement_display,
                    handle_confirm_settlement,
                    handle_close_settlement_modal,
                    despawn_settlement_modal,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(
                Update,
                (
                    ai_create_trade_offers,
                    ai_accept_trade_offers,
                    ai_settle_trades,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(Update, button_action)
            .add_observer(offer_published)
            .add_observer(can_trade_removed);
    }
}
