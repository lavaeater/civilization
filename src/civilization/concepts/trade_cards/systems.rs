use crate::civilization::components::general_components::PlayerCities;
use crate::civilization::concepts::trade_cards::components::{CivilizationTradeCards, PlayerTradeCards};
use crate::civilization::concepts::trade_cards::events::{CheckIfWeCanTrade, HumanPlayerPulledTradeCard};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{debug, Entity, EventReader, EventWriter, Has, NextState, Query, ResMut};

pub fn acquire_trade_cards(
    mut player_query: Query<(Entity, &PlayerCities, &mut PlayerTradeCards, Has<IsHuman>)>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    mut check_if_we_can_trade: EventWriter<CheckIfWeCanTrade>,
    mut pulled_card_event_writer: EventWriter<HumanPlayerPulledTradeCard>
) {
    for (player_entity, player_cities, mut player_trade_cards, is_human) in player_query
        .iter_mut()
        .sort_by::<&PlayerCities>(|v1, v2| {
            v1.number_of_cities()
                .cmp(&v2.number_of_cities())
        }) {
//        player_cities.number_of_cities() +1
        // for now, we pull trade cards every round because why not?
        (1..=player_cities.number_of_cities()).for_each(|pile| {
            if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                debug!("Player acquired trade card: {:?}", pulled_card);
                player_trade_cards.add_trade_card(pulled_card);
                pulled_card_event_writer.send(HumanPlayerPulledTradeCard::new(player_entity));
            } else {
                debug!("No more trade cards in pile: {}", pile);
            }
        });
    }
    check_if_we_can_trade.send(CheckIfWeCanTrade);
}

pub fn transition_to_trade(
    mut check_if_we_can_trade: EventReader<CheckIfWeCanTrade>,
    players_can_trade_query: Query<(&PlayerTradeCards, Has<IsHuman>)>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in check_if_we_can_trade.read() {
        if players_can_trade_query.iter().filter(|(trade, _)| trade.can_trade()).count() >= 2
            && players_can_trade_query.iter().filter(|(_, is_human)| *is_human).count() > 0 {
            debug!("COMMENCE TRADING!");
            next_state.set(GameActivity::Trade);
        } else {
            debug!("Not enough players can trade");
            next_state.set(GameActivity::PopulationExpansion);
        }
    }
}


