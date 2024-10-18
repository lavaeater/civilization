use bevy::prelude::{debug, NextState, Query, ResMut};
use crate::civilization::general::general_components::PlayerCities;
use crate::civilization::trade_cards::trade_card_components::PlayerTradeCards;
use crate::civilization::trade_cards::trade_card_components::CivilizationTradeCards;
use crate::GameActivity;

pub fn acquire_trade_cards(
    mut player_query: Query<(&PlayerCities, &mut PlayerTradeCards)>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    mut next_state: ResMut<NextState<GameActivity>>
) {
    for (player_cities, mut player_trade_cards) in player_query
        .iter_mut()
        .sort_by::<&PlayerCities>(|v1, v2| {
            v1.number_of_cities()
                .cmp(&v2.number_of_cities())
        }) {
        (1..player_cities.number_of_cities() +1).for_each(|pile| {
            if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                debug!("Player acquired trade card: {:?}", pulled_card);
                player_trade_cards.add_trade_card(pulled_card);
            } else {
                debug!("No more trade cards in pile: {}", pile);
            }
        });
    }
    next_state.set(GameActivity::Trade);
}

