use crate::civilization::components::prelude::PlayerTradeCards;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{Color, Commands, Entity, Has, Local, Name, NextState, PositionType, Query, ResMut, Val};
use sickle_ui::prelude::*;
use sickle_ui::ui_commands::*;

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
            // Let's create a simple column widget on the screen.
            commands.ui_builder(UiRoot).column(|column| {
                // We can style our widget directly in code using the style method.
                column
                    .style()
                    // The column will be located 100 pixels from the right and 100 pixels from the top of the screen.
                    // The absolute position means we are not set relative to any parent.
                    .position_type(PositionType::Absolute)
                    .right(Val::Px(100.0))
                    .top(Val::Px(100.0))
                    // We'll bound the height of our column to the total height of our contents.
                    // By default, a column will be 100% of the parent's height which would be the entire length of the screen.,
                    .height(Val::Auto)
                    // Lets give it a visible background color.
                    .background_color(Color::srgb(0.5, 0.5, 0.5));

                // Let's add some content to our column.
                column
                    .label(LabelConfig::default())
                    .entity_commands()
                    // We can use the set_text method to set the text of a label.
                    .set_text("This is label 1.", None);

                column
                    .label(LabelConfig::default())
                    .entity_commands()
                    .set_text("This is another label.", None);
            });
        }
        // if let Ok(camera) = camera_query.get_single() {

        // if let Ok((player, player_name, _)) = human_player_query.get_single() {
        //     commands.spawn(UserTradeMenu {
        //         player,
        //         player_name: player_name.clone(),
        //     }.to_root());
        // }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}