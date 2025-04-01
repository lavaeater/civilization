use crate::civilization::concepts::trade::components::{
    PublishedOffer, PublishedOffersList, TradeOffer,
};
use crate::civilization::ui::ui_builder::{ButtonDef, NodePartial, UIBuilder};
use bevy::color::palettes::basic::{GREEN, WHITE, YELLOW};
use bevy::color::palettes::css::{DARK_GRAY, DARK_GREEN, DARK_RED};
use bevy::prelude::{default, AlignItems, AssetServer, Color, Commands, Entity, JustifyContent, OnAdd, Query, Res, Trigger, UiRect, Val, With};
use bevy::ui::BorderRadius;

pub fn offer_published(
    trigger: Trigger<OnAdd, PublishedOffer>,
    published_offers_list: Query<Entity, With<PublishedOffersList>>,
    published_offer_query: Query<&TradeOffer, With<PublishedOffer>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let new_commands = commands;
    if let Ok(trade_offer) = published_offer_query.get(trigger.entity()) {
        if let Ok(ui_list) = published_offers_list.get_single() {
            let font = asset_server.load("fonts/FiraSans-Bold.ttf");
            let card_color = Color::srgba(0.7, 0.6, 0.2, 0.8);
            let bg_color = Color::srgba(0.5, 0.5, 0.5, 0.25);
            let border_color = Color::srgba(0.2, 0.2, 0.2, 0.8);
            
            let mut ui_builder = UIBuilder::start_from_entity(new_commands, ui_list, false);
            
            ui_builder
                .child()
                .as_block(Val::Percent(100.0), Val::Auto, bg_color)
                .with_margin(UiRect::all(Val::Px(10.0)))
                .with_padding(UiRect::all(Val::Px(10.0)))
                .with_border(UiRect::all(Val::Px(2.0)), border_color)
                
                // Add header with initiator name
                .child()
                .as_flex_row()
                .with_justify_content(JustifyContent::SpaceBetween)
                .with_size(Val::Percent(100.0), Val::Auto)
                .with_margin(UiRect::bottom(Val::Px(10.0)))
                
                // Add initiator name
                .child()
                .with_text(
                    format!("Trade offer from: {}", trade_offer.initiator_name),
                    font.clone(),
                    24.0,
                    Some(Color::WHITE),
                )
                .parent()
                
                // Add status indicators (open/direct/accepted)
                .child()
                .with_text(
                    if trade_offer.receiver.is_none() { "Open Offer" } 
                    else if trade_offer.trade_accepted() { "Accepted" }
                    else { "Direct Offer" },
                    font.clone(),
                    20.0,
                    Some(if trade_offer.trade_accepted() { Color::from(GREEN) } else { Color::from(YELLOW) }),
                )
                .parent()
                .parent()
                
                // Trade details section
                .child()
                .as_flex_row()
                .with_size(Val::Percent(100.0), Val::Auto)
                .with_justify_content(JustifyContent::SpaceBetween)
                
                // What initiator offers
                .child()
                .as_flex_col_with_props(Val::Percent(48.0), Val::Auto, card_color)
                .with_padding(UiRect::all(Val::Px(8.0)))
                
                // Header for what initiator offers
                .child()
                .with_text(
                    "Offers:",
                    font.clone(),
                    20.0,
                    Some(Color::WHITE),
                )
                .parent()
                .with_children(|builder| {
                for (commodity, count) in trade_offer.initiator_pays.iter() {
                    builder
                        .child()
                        .with_text(
                            format!("{}: {}", commodity, count),
                            font.clone(),
                            18.0,
                            Some(Color::WHITE),
                        )
                        .parent();
                }
            })
                .with_children(|builder| {
                    for (commodity, count) in trade_offer.initiator_pays_guaranteed.iter() {
                        builder
                            .child()
                            .with_text(
                                format!("{}: {} (Guaranteed)", commodity, count),
                                font.clone(),
                                18.0,
                                Some(Color::from(GREEN)),
                            )
                            .parent();
                    }
                })
                .parent()
                .child()
                .as_flex_col_with_props(Val::Percent(48.0), Val::Auto, card_color)
                .with_padding(UiRect::all(Val::Px(8.0)))
                
                // Header for what initiator wants
                .child()
                .with_text(
                    "Wants:",
                    font.clone(),
                    20.0,
                    Some(Color::WHITE),
                )
                .parent()
                .with_children(|builder| {
                    for (commodity, count) in trade_offer.initiator_gets.iter() {
                    builder
                        .child()
                        .with_text(
                            format!("{}: {}", commodity, count),
                            font.clone(),
                            18.0,
                            Some(Color::from(WHITE)),
                        )
                        .parent();
                }})
                .with_children(|builder| {for (commodity, count) in trade_offer.initiator_gets_guaranteed.iter() {
                    builder
                        .child()
                        .with_text(
                            format!("{}: {} (Guaranteed)", commodity, count),
                            font.clone(),
                            18.0,
                            Some(Color::from(GREEN)),
                        )
                        .parent();
                }});
                
            // Add initiator gets items
            
            
            // Add guaranteed items with special marking
            
            
            // Add action buttons if this is a direct offer to the player
            if let Some(receiver) = trade_offer.receiver {
                ui_builder
                    .parent()
                    .parent()
                    .child()
                    .as_flex_row()
                    .with_size(Val::Percent(100.0), Val::Px(40.0))
                    .with_justify_content(JustifyContent::FlexEnd)
                    .with_margin(UiRect::top(Val::Px(10.0)))
                    
                    // Accept button
                    .child()
                    .with_button(ButtonDef {
                        text: "Accept".to_string(),
                        font: font.clone_weak(),
                        font_size: 18.0,
                        text_color: Color::WHITE,
                        bg_color,
                        border_color,
                        border_radius: BorderRadius::MAX,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .as_block(Val::Px(100.0), Val::Px(30.0), Color::from(DARK_GREEN))
                    .with_margin(UiRect::right(Val::Px(10.0)))
                    .with_button()
                    .with_text(
                        "Accept",
                        font.clone(),
                        18.0,
                        Some(Color::WHITE),
                    )
                    .parent()
                    
                    // Counter button
                    .child()
                    .as_block(Val::Px(100.0), Val::Px(30.0), Color::from(DARK_GRAY))
                    .with_margin(UiRect::right(Val::Px(10.0)))
                    .with_button()
                    .with_text(
                        "Counter",
                        font.clone(),
                        18.0,
                        Some(Color::WHITE),
                    )
                    .parent()
                    
                    // Decline button
                    .child()
                    .as_block(Val::Px(100.0), Val::Px(30.0), Color::from(DARK_RED))
                    .with_button()
                    .with_text(
                        "Decline",
                        font.clone(),
                        18.0,
                        Some(Color::WHITE),
                    );
            }
            
            // Build the UI
            commands = ui_builder.build().1;
        }
    }
}
