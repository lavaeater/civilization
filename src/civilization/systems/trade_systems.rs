use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::resources::prelude::TradeOffers;
use crate::civilization::ui::ui_plugin::{style_row, style_test};
use crate::GameActivity;
use bevy::log::info;
use bevy::prelude::{Camera, Commands, Entity, Label, NextState, NodeBundle, Query, ResMut, TargetCamera, Text, TextStyle, With};
use bevy_quill::{Cx, Element, For, View, ViewTemplate};
use bevy_quill_obsidian::controls::{Button, ButtonVariant};
use crate::stupid_ai::stupid_ai_components::StupidAi;

impl ViewTemplate for TradeOffersList {
    type View = impl View;
    fn create(&self, cx: &mut Cx) -> Self::View {
        let offers = cx.use_resource::<TradeOffers>().offers.iter().map(|offer| { "Offer" }).collect::<Vec<&str>>();
        Element::<NodeBundle>::new()
            .children(("Trade Offers",
                       For::each(offers, |offer| {
                           (String::from(*offer),
                           )
                       }).with_fallback("No items")))
    }
}

#[derive(Clone, PartialEq)]
struct UserTradeMenu {
    camera: Entity,
}

impl ViewTemplate for UserTradeMenu {
    type View = impl View;
    fn create(&self, cx: &mut Cx) -> Self::View {
        let mut trade_offer_resource = cx.use_resource::<TradeOffers>();
        let offers = trade_offer_resource
            .offers
            .iter()
            .map(|offer| { "Offer" })
            .collect::<Vec<&str>>();

        let click = cx.create_callback(|| {
            trade_offer_resource.create_new_offer();
        });
        Element::<NodeBundle>::new()
            .insert_dyn(TargetCamera, self.camera)
            .style(style_test)
            .children((
                "Trade Offers",
                Element::<NodeBundle>::new().style(style_row).children((
                    Button::new().on_click(click).children("Default"),
                    Button::new()
                        .on_click(click)
                        .variant(ButtonVariant::Primary)
                        .children("Primary"),
                    Button::new()
                        .on_click(click)
                        .variant(ButtonVariant::Danger)
                        .children("Danger"),
                    Button::new()
                        .on_click(click)
                        .variant(ButtonVariant::Selected)
                        .children("Selected"),
                    Button::new().minimal(true).children("Minimal"),
                )),
            ))
    }
}

pub fn setup_basic_trade_ui(
    mut commands: Commands,
    players_can_trade_query: Query<(&PlayerTradeCards, Has<StupidAi)>>,
    camera_query: Query<Entity, With<Camera>>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if players_can_trade_query.iter().filter(|trade| trade.can_trade()).count() >= 2 {
        if let Ok(camera) = camera_query.get_single() {
            commands.spawn(UserTradeMenu { camera }.to_root());
        }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}