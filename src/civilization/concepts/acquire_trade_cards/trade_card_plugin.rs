use crate::civilization::concepts::acquire_trade_cards::trade_card_events::{
    CheckIfWeCanTrade, HumanPlayerTradeCardsUpdated,
};
use crate::civilization::concepts::acquire_trade_cards::trade_card_systems::{
    acquire_trade_cards, transition_to_trade,
};
use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoScheduleConfigs, OnEnter, Plugin, Update};
use crate::civilization::concepts::acquire_trade_cards::trade_card_components::CivilizationTradeCards;

pub struct TradeCardPlugin;

impl Plugin for TradeCardPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CivilizationTradeCards::new())
        .add_message::<CheckIfWeCanTrade>()
        .add_message::<HumanPlayerTradeCardsUpdated>()
        .add_systems(
            OnEnter(GameActivity::AcquireTradeCards),
            acquire_trade_cards,
        )
        .add_systems(
            Update,
            transition_to_trade.run_if(in_state(GameActivity::AcquireTradeCards)),
        );
    }
}
