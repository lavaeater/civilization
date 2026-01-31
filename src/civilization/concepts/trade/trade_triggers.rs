use crate::civilization::concepts::trade::trade_components::{
    CanTrade, PlayerTradeInterests, PublishedOffer, PublishedOffersList, TradeButtonAction,
    TradeOffer,
};
use crate::civilization::ui::ui_builder::{
    ButtonPartial, UIBuilder, UiBuilderDefaults, BG_COLOR, BORDER_COLOR,
};
use crate::GameActivity;
use bevy::color::palettes::basic::{GREEN, YELLOW};
use bevy::log::debug;
use bevy::prelude::{
    default, Color, Commands, Entity, JustifyContent, NextState, Add, Remove, Query, Res,
    ResMut, On, UiRect, Val, With,
};
use crate::civilization::game_moves::game_moves_components::TradeMove;

#[allow(unused_assignments)]
pub fn offer_published(
    trigger: On<Add, PublishedOffer>,
    published_offers_list: Query<Entity, With<PublishedOffersList>>,
    published_offer_query: Query<(Entity, &TradeOffer), With<PublishedOffer>>,
    ui_builder_defaults: Res<UiBuilderDefaults>,
    mut commands: Commands,
) {
    let new_commands = commands;
    if let Ok((trade_offer_entity, trade_offer)) = published_offer_query.get(trigger.event().entity) {
        if let Ok(ui_list) = published_offers_list.single() {
            let mut ui_builder = UIBuilder::start_from_entity(
                new_commands,
                ui_list,
                false,
                Some(ui_builder_defaults.clone()),
            );

            ui_builder
                .child()
                .as_block(Val::Percent(100.0), Val::Auto, BG_COLOR)
                .margin(UiRect::all(Val::Px(10.0)))
                .padding(UiRect::all(Val::Px(10.0)))
                .border(UiRect::all(Val::Px(2.0)), BORDER_COLOR)
                // Add header with initiator name
                .child()
                .as_flex_row()
                .justify_content(JustifyContent::SpaceBetween)
                .size(Val::Percent(100.0), Val::Auto)
                .margin(UiRect::bottom(Val::Px(10.0)))
                // Add initiator name
                .child()
                .default_text(format!("Trade offer from: {}", trade_offer.initiator_name))
                .parent()
                // Add status indicators (open/direct/accepted)
                .child()
                .with_text(
                    if trade_offer.trade_accepted() {
                        "Accepted"
                    } else {
                        "Direct Offer"
                    },
                    None,
                    None,
                    Some(if trade_offer.trade_accepted() {
                        Color::from(GREEN)
                    } else {
                        Color::from(YELLOW)
                    }),
                )
                .parent()
                .parent()
                // Trade details section
                .child()
                .as_flex_row()
                .size(Val::Percent(100.0), Val::Auto)
                .justify_content(JustifyContent::SpaceBetween)
                // What initiator offers
                .child()
                .as_flex_col(Val::Percent(48.0), Val::Auto)
                .padding(UiRect::all(Val::Px(8.0)))
                // Header for what initiator offers
                .child()
                .default_text("Offers:")
                .parent()
                .foreach_child(trade_offer.initiator_pays.iter(), |builder, (commodity, count)| {
                    builder.default_text(format!("{}: {}", commodity, count));
                })
                .foreach_child(
                    trade_offer.initiator_pays_guaranteed.iter(),
                    |builder, (commodity, count)| {
                        builder.default_text(format!(
                            "{}: {} (Guaranteed)",
                            commodity, count
                        ));
                    },
                )
                .parent()
                .child()
                .as_flex_col(Val::Percent(48.0), Val::Auto)
                .padding(UiRect::all(Val::Px(8.0)))
                // Header for what initiator wants
                .child()
                .default_text("Wants:")
                .parent()
                .foreach_child(trade_offer.initiator_gets.iter(), |builder, (commodity, count)| {
                    builder.default_text(format!("{}: {}", commodity, count));
                })
                .foreach_child(
                    trade_offer.initiator_gets_guaranteed.iter(),
                    |builder, (commodity, count)| {
                        builder.default_text(format!(
                            "{}: {} (Guaranteed)",
                            commodity, count
                        ));
                    },
                );

            ui_builder
                    .parent()
                    .parent()
                    .child()
                    .as_flex_row()
                    .size(Val::Percent(100.0), Val::Px(40.0))
                    .justify_content(JustifyContent::FlexEnd)
                    .margin(UiRect::top(Val::Px(10.0)))

                    // Accept button
                    .with_button(Some(ButtonPartial {
                        text: Some("Accept".to_string()),
                        ..default()
                    }), TradeButtonAction::TradeAction(TradeMove::AcceptOrDeclineTrade(trade_offer_entity)))
            
                // Decline button
                ;

            // Build the UI
            commands = ui_builder.build().1;
        }
    }
}

pub fn can_trade_removed(
    _trigger: On<Remove, CanTrade>,
    players_can_trade: Query<(Entity, &CanTrade)>,
    trade_offers: Query<(Entity, &TradeOffer)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    debug!("Someone decided not to trade anymore.");
    debug!("{} players can trade", players_can_trade.iter().len());
    if players_can_trade.iter().len() < 2 {
        debug!("Not enough players can trade. Skipping trade phase.");
        for (entity, _) in players_can_trade.iter() {
            commands.entity(entity).remove::<CanTrade>();
            commands.entity(entity).remove::<PlayerTradeInterests>();
        }
        for (entity, _) in trade_offers.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameActivity::PopulationExpansion);
    }
}
