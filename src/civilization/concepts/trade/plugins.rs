use crate::civilization::concepts::trade::events::SendTradingCardsCommand;
use crate::civilization::concepts::trade::resources::{TradeCountdown, TradeUiState};
use crate::civilization::systems::prelude::{begin_trade_settlement, button_action, delay_trade_moves_if_offers_are_accepted, handle_send_trading_cards_command, remove_rejected_trades, setup_trade, trigger_trade_moves};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin, Update};
use crate::civilization::concepts::trade::triggers::offer_published;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TradeUiState::default()) // Placeholder until actual resources are added
            .init_resource::<TradeCountdown>()
            .add_event::<SendTradingCardsCommand>()
            .add_systems(OnEnter(GameActivity::Trade), setup_trade)
            .add_systems(
                Update,
                (
                    button_action,
                    trigger_trade_moves,
                    remove_rejected_trades, 
                    delay_trade_moves_if_offers_are_accepted,
                    begin_trade_settlement,
                    handle_send_trading_cards_command
                )
                    .run_if(in_state(GameActivity::Trade)),
            ).add_observer(offer_published)
        ;
    }
}
