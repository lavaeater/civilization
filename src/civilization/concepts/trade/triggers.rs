use crate::civilization::concepts::trade::components::{
    PublishedOffer, PublishedOffersList, TradeOffer,
};
use crate::civilization::ui::ui_builder::UIBuilder;
use bevy::prelude::{Commands, Entity, OnAdd, Query, Trigger, With};

pub fn offer_published(
    trigger: Trigger<OnAdd, PublishedOffer>,
    published_offers_list: Query<Entity, With<PublishedOffersList>>,
    published_offer_query: Query<&TradeOffer, With<PublishedOffer>>,
    commands: Commands,
) {
    if let Ok(trade_offer) = published_offer_query.get(trigger.entity()) {
        if let Ok(ui_list) = published_offers_list.get_single() {
            UIBuilder::start_from_entity(commands, ui_list, false)
                .move_to_new_child()
            ;
        }
    }
}
