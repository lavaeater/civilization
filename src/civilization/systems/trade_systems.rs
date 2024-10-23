use crate::civilization::components::prelude::PlayerTradeCards;
use crate::civilization::resources::prelude::TradeOffers;
use crate::civilization::ui::ui_plugin::{style_row, style_test};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, Has, NextState, NodeBundle, Query, ResMut, World};
use bevy_quill::{Cx, Element, View, ViewTemplate};
use bevy_quill_obsidian::controls::{Button, ButtonVariant};

#[derive(Clone, PartialEq)]
struct UserTradeMenu {
    player: Entity
}

impl ViewTemplate for UserTradeMenu {
    type View = impl View;
    fn create(&self, cx: &mut Cx) -> Self::View {
        let trade_offer_resource = cx.use_resource::<TradeOffers>();
        let offers = trade_offer_resource
            .offers
            .iter()
            .map(|offer| { "Offer" })
            .collect::<Vec<&str>>();
        let p = self.player;
        let click = cx.create_callback(move |world: &mut World| {
            let mut trade_offer_resource = world.get_resource_mut::<TradeOffers>().unwrap();
            trade_offer_resource.create_new_offer(p);
        });
        Element::<NodeBundle>::new()
            // .insert_dyn(TargetCamera, self.camera)
            .style(style_test)
            .children((
                "Trade Offers",
                Element::<NodeBundle>::new().style(style_row).children((
                    Button::new()
                        .on_click(click)
                        .variant(ButtonVariant::Primary)
                        .children("New Offer"),
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
    players_can_trade_query: Query<(&PlayerTradeCards, Has<IsHuman>)>,
    human_player_query: Query<(Entity, &IsHuman)>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if players_can_trade_query.iter().filter(|(trade, _)| trade.can_trade()).count() >= 2 
        && players_can_trade_query.iter().filter(|(_, is_human)| *is_human).count() == 1 {
        // if let Ok(camera) = camera_query.get_single() {
            
            commands.spawn(UserTradeMenu{player: human_player_query.get_single().unwrap().0}.to_root());
        // }
    } else {
        next_state.set(GameActivity::PopulationExpansion);
    }
}