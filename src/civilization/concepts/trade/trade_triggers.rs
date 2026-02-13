use crate::civilization::concepts::trade::trade_components::{
    CanTrade, PublishedOffer, PublishedOffersList, TradeButtonAction,
    TradeOffer,
};
use lava_ui_builder::UIBuilder;
use lava_ui_builder::UiTheme;
use crate::GameActivity;
use bevy::color::palettes::basic::{GREEN, YELLOW};
use bevy::log::debug;
use bevy::prelude::{
    Add, Color, Commands, Entity, JustifyContent, NextState, On, Query, Remove,
    Res, ResMut, UiRect, Val, With,
};
use crate::civilization::TradeMove;

#[allow(unused_assignments)]
pub fn offer_published(
    trigger: On<Add, PublishedOffer>,
    published_offers_list: Query<Entity, With<PublishedOffersList>>,
    published_offer_query: Query<(Entity, &TradeOffer), With<PublishedOffer>>,
    ui_theme: Res<UiTheme>,
    mut commands: Commands,
) {
    let new_commands = commands;
    if let Ok((trade_offer_entity, trade_offer)) = published_offer_query.get(trigger.event().entity) {
        if let Ok(ui_list) = published_offers_list.single() {
            let mut ui_builder = UIBuilder::start_from_entity(
                new_commands,
                ui_list,
                false,
                Some(ui_theme.clone()),
            );

            let border_color = ui_theme.border_color;
            let bg_color = ui_theme.bg_color;
            ui_builder
                .child()
                .size(Val::Percent(100.0), Val::Auto)
                .bg_color(bg_color)
                .margin(UiRect::all(Val::Px(10.0)))
                .padding(UiRect::all(Val::Px(10.0)))
                .border(UiRect::all(Val::Px(2.0)), border_color)
                // Add header with initiator name
                .child()
                .flex_row()
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
                    None, None
                )
                .parent()
                .parent()
                // Trade details section
                .child()
                .flex_row()
                .size(Val::Percent(100.0), Val::Auto)
                .justify_content(JustifyContent::SpaceBetween)
                // What initiator offers
                .child()
                .display_flex().flex_column().size(Val::Percent(48.0), Val::Auto)
                .padding(UiRect::all(Val::Px(8.0)))
                // Header for what initiator offers
                .child()
                .default_text("Offers:")
                .parent()
                .foreach_child(trade_offer.initiator_pays.iter(), |builder: &mut UIBuilder, (commodity, count)| {
                    builder.default_text(format!("{}: {}", commodity, count));
                })
                .foreach_child(
                    trade_offer.initiator_pays_guaranteed.iter(),
                    |builder: &mut UIBuilder, (commodity, count)| {
                        builder.default_text(format!(
                            "{}: {} (Guaranteed)",
                            commodity, count
                        ));
                    },
                )
                .parent()
                .child()
                .display_flex().flex_column().size(Val::Percent(48.0), Val::Auto)
                .padding(UiRect::all(Val::Px(8.0)))
                // Header for what initiator wants
                .child()
                .default_text("Wants:")
                .parent()
                .foreach_child(trade_offer.initiator_gets.iter(), |builder: &mut UIBuilder, (commodity, count)| {
                    builder.default_text(format!("{}: {}", commodity, count));
                })
                .foreach_child(
                    trade_offer.initiator_gets_guaranteed.iter(),
                    |builder: &mut UIBuilder, (commodity, count)| {
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
                    .flex_row()
                    .size(Val::Percent(100.0), Val::Px(40.0))
                    .justify_content(JustifyContent::FlexEnd)
                    .margin(UiRect::top(Val::Px(10.0)))

                    // Accept button
                    .add_themed_button(
                        TradeButtonAction::TradeAction(TradeMove::AcceptOrDeclineTrade(trade_offer_entity)),
                        |btn| { btn.text("Accept"); },
                    )
            
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
    debug!("{} players can still trade", players_can_trade.iter().len());
    // Only end trading when NO players remain. The countdown timer
    // handles removing CanTrade from everyone when it expires.
    if players_can_trade.iter().len() == 0 {
        debug!("No players left trading. Ending trade phase.");
        for (entity, _) in trade_offers.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameActivity::PopulationExpansion);
    }
}
