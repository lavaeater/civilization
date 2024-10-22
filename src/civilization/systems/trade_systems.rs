use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::resources::prelude::TradeOffers;
use crate::civilization::ui::ui_plugin::{style_row, style_test};
use crate::GameActivity;
use bevy::log::info;
use bevy::prelude::{Camera, Commands, Entity, Label, NextState, NodeBundle, Query, ResMut, TargetCamera, With};
use bevy_quill::{Cx, Element, For, View, ViewTemplate};
use bevy_quill_obsidian::controls::{Button, ButtonVariant};

#[derive(Clone, PartialEq)]
struct TradeOffersList {
    camera: Entity,
}

impl ViewTemplate for TradeOffersList {
    type View = impl View;
    fn create(&self, cx: &mut Cx) -> Self::View {
        let offers = cx.use_resource::<TradeOffers>();
        Element::<NodeBundle>::new()
            .children(For::each(&offers.offers, |item| {
                (
                    "Offers",
                    Element::<NodeBundle>::new().style(style_row).children((
                        Label,
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
                )
            }).with_fallback("No items")),
    }
}

#[derive(Clone, PartialEq)]
struct UserTradeMenu {
    camera: Entity,
}

impl ViewTemplate for UserTradeMenu {
    type View = impl View;
    fn create(&self, cx: &mut Cx) -> Self::View {
        let click = cx.create_callback(|| {
            info!("Clicked!");
        });
        Element::<NodeBundle>::new()
            .insert_dyn(TargetCamera, self.camera)
            .style(style_test)
            .children((
                "Variants",
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
    players_can_trade_query: Query<&PlayerTradeCards>,
    camera_query: Query<Entity, With<Camera>>,
    mut next_state: ResMut<NextState<GameActivity>>
) {
    if players_can_trade_query.iter().filter(|trade| trade.can_trade()).count() >= 2 {
        if let Ok(camera) = camera_query.get_single() {
            commands.spawn(UserTradeMenu { camera }.to_root());
        }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}