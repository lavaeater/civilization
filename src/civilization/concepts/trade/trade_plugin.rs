use crate::GameActivity;
use crate::civilization::concepts::trade::trade_events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::trade_inbox::{
    TradeInbox, collect_inbox, handle_inbox_buttons, handle_table_log_toggle, inbox_shortcuts,
    render_inbox_card, render_my_deals, render_table_log,
};
use crate::civilization::concepts::trade::trade_resources::{
    CreateOfferState, TradeCountdown, TradePhaseState, TradeUiState,
};
use crate::civilization::concepts::trade::trade_systems::*;
use crate::civilization::concepts::trade::trade_table_ui::{
    despawn_talk_composer, handle_proposal_action_buttons, handle_say_button,
    handle_talk_composer_buttons, handle_talk_line_buttons, spawn_talk_composer,
    update_talk_composer,
};
use crate::civilization::concepts::trade::trade_talk::{
    TalkDraft, ai_trade_talk, clear_withdrawn_offers, log_new_talk, prune_old_talk,
    withdraw_unfulfillable_proposals,
};
use crate::civilization::concepts::trade::trade_triggers::{can_trade_removed, offer_published};
use bevy::app::App;
use bevy::prelude::{IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update, in_state};

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TradeUiState::default())
            .init_resource::<TradeCountdown>()
            .init_resource::<TradePhaseState>()
            .init_resource::<CreateOfferState>()
            .init_resource::<TalkDraft>()
            .init_resource::<TradeInbox>()
            .add_message::<SendTradingCardsCommand>()
            .add_systems(
                OnEnter(GameActivity::Trade),
                (setup_trade, setup_trade_phase_ui),
            )
            .add_systems(OnExit(GameActivity::Trade), cleanup_trade_phase_ui)
            .add_systems(
                Update,
                (
                    trigger_trade_moves,
                    remove_rejected_trades,
                    delay_trade_moves_if_offers_are_accepted,
                    begin_trade_settlement,
                    handle_send_trading_cards_command,
                    settle_trades,
                    check_trade_gate,
                    update_trade_countdown_display,
                    update_player_trading_status,
                    handle_done_trading_button,
                    handle_create_offer_button,
                    handle_accept_offer_button,
                    update_human_done_ui,
                    refresh_player_cards_display,
                    handle_decline_offer_button,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(
                Update,
                (
                    spawn_create_offer_modal,
                    handle_close_create_offer_modal,
                    despawn_create_offer_modal,
                    handle_card_count_adjust,
                    update_card_count_displays,
                    handle_hidden_count_buttons,
                    update_hidden_count_displays,
                    handle_claim_buttons,
                    update_claim_display,
                    update_offer_summary_display,
                    handle_publish_offer,
                    handle_target_buttons,
                    update_target_display,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(
                Update,
                (
                    handle_say_button,
                    handle_talk_line_buttons,
                    handle_proposal_action_buttons,
                    spawn_talk_composer,
                    handle_talk_composer_buttons,
                    update_talk_composer,
                    despawn_talk_composer,
                )
                    .chain()
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_systems(
                Update,
                (
                    collect_inbox,
                    inbox_shortcuts,
                    handle_inbox_buttons,
                    render_inbox_card,
                    render_my_deals,
                    handle_table_log_toggle,
                    render_table_log,
                )
                    .chain()
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
                    finalize_settled_open_offers,
                    ai_stop_trading_when_ready,
                    ai_trade_talk,
                    withdraw_unfulfillable_proposals,
                    clear_withdrawn_offers,
                    prune_old_talk,
                    log_new_talk,
                )
                    .run_if(in_state(GameActivity::Trade)),
            )
            .add_observer(offer_published)
            .add_observer(can_trade_removed);
    }
}
