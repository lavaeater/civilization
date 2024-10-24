use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::resources::prelude::TradeResources;
use crate::civilization::ui::ui_plugin::{style_row, style_test};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, Has, Name, NextState, NodeBundle, Query, ResMut, World};

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
) {
    if players_can_trade_query.iter().filter(|(trade, _)| trade.can_trade()).count() >= 2
        && players_can_trade_query.iter().filter(|(_, is_human)| *is_human).count() == 1 {
        // if let Ok(camera) = camera_query.get_single() {

        if let Ok((player, player_name, _)) = human_player_query.get_single() {
            commands.spawn(UserTradeMenu {
                player,
                player_name: player_name.clone(),
            }.to_root());
        }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}