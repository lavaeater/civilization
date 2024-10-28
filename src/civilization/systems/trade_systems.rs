use crate::civilization::components::prelude::PlayerTradeCards;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, Has, Local, Name, NextState, Query, ResMut};

#[derive(Clone, PartialEq)]
struct UserTradeMenu {
    player: Entity,
    player_name: Name,
}
pub fn setup_human_trading_ui(
    mut commands: Commands,
    players_can_trade_query: Query<(&PlayerTradeCards, Has<IsHuman>)>,
    human_player_query: Query<(Entity, &Name, &IsHuman)>,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut already_has_ui: Local<bool>
) {
    if players_can_trade_query.iter().filter(|(trade, _)| trade.can_trade()).count() >= 2
        && players_can_trade_query.iter().filter(|(_, is_human)| *is_human).count() == 1 {
        if !*already_has_ui {
            *already_has_ui = true;
        }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}