use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::ui::ui_plugin::{style_row, style_test};
use crate::stupid_ai::stupid_ai_components::StupidAi;
use crate::GameActivity;
use bevy::prelude::{Commands, Has, NextState, NodeBundle, Query, ResMut};
use bevy_quill::{Cx, Element, View, ViewTemplate};
use bevy_quill_obsidian::controls::{Button, ButtonVariant};

#[derive(Clone, PartialEq)]
struct UserTradeMenu {
}

impl ViewTemplate for UserTradeMenu {
    type View = impl View;
    fn create(&self, cx: &mut Cx) -> Self::View {
        // let mut trade_offer_resource = cx.use_resource::<TradeOffers>();
        // let offers = trade_offer_resource
        //     .offers
        //     .iter()
        //     .map(|offer| { "Offer" })
        //     .collect::<Vec<&str>>();

        let click = cx.create_callback(|| {
            // trade_offer_resource.create_new_offer();
        });
        Element::<NodeBundle>::new()
            // .insert_dyn(TargetCamera, self.camera)
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
        // Element::<NodeBundle>::new()
        //     .children(("Trade Offers",
        //                For::each(offers, |offer| {
        //                    (String::from(*offer),
        //                    )
        //                }).with_fallback("No items")))
    }
}

pub fn setup_human_trading_ui(
    mut commands: Commands,
    players_can_trade_query: Query<(&PlayerTradeCards, Has<StupidAi>)>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if players_can_trade_query.iter().filter(|(trade, _)| trade.can_trade()).count() >= 2 { 
        // &&
        // players_can_trade_query.iter().filter(|(_, is_ai)| !is_ai).count() == 1 {
        // if let Ok(camera) = camera_query.get_single() {
            
            commands.spawn(UserTradeMenu{}.to_root());
        // }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}