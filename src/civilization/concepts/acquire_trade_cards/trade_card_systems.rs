use crate::civilization::components::Faction;
use crate::civilization::components::PlayerCities;
use crate::civilization::concepts::acquire_trade_cards::trade_card_components::{
    CivilizationTradeCards, PlayerTradeCards,
};
use crate::civilization::concepts::acquire_trade_cards::trade_card_events::{
    CheckIfWeCanTrade, HumanPlayerTradeCardsUpdated,
};
use crate::civilization::plugins::civilization_plugin::DebugOptions;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::{debug, Entity, MessageReader, MessageWriter, Has, NextState, Query, Res, ResMut};

pub fn acquire_trade_cards(
    mut player_query: Query<(
        Entity,
        &Faction,
        &PlayerCities,
        &mut PlayerTradeCards,
        Has<IsHuman>,
    )>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    mut check_if_we_can_trade: MessageWriter<CheckIfWeCanTrade>,
    mut pulled_card_event_writer: MessageWriter<HumanPlayerTradeCardsUpdated>,
    debug_options: Res<DebugOptions>,
) {
    for (player_entity, faction, player_cities, mut player_trade_cards, is_human) in player_query
        .iter_mut()
        .sort_by::<&PlayerCities>(|v1, v2| v1.number_of_cities().cmp(&v2.number_of_cities()))
    {
        let mut pulled_cards = false;
        if is_human && debug_options.human_always_pulls_trade_cards {
            (1..=player_cities.number_of_cities() + 1).for_each(|pile| {
                if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                    pulled_cards = true;
                    player_trade_cards.add_trade_card(pulled_card);
                } else {
                    debug!("No more trade cards in pile: {}", pile);
                }
            });
        } else if !is_human && debug_options.ai_always_pulls_trade_cards {
            debug!("AI ALWAYS PULLS TRADE CARDS");
            (1..=player_cities.number_of_cities() + 3).for_each(|pile| {
                if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                    pulled_cards = true;
                    player_trade_cards.add_trade_card(pulled_card);
                } else {
                    debug!("No more trade cards in pile: {}", pile);
                }
            });
        } else {
            (1..=player_cities.number_of_cities()).for_each(|pile| {
                if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                    pulled_cards = true;
                    player_trade_cards.add_trade_card(pulled_card);
                } else {
                    debug!("No more trade cards in pile: {}", pile);
                }
            });
        }
        if is_human && pulled_cards {
            pulled_card_event_writer.write(HumanPlayerTradeCardsUpdated::new(player_entity));
        }
        if pulled_cards {
            debug!(
                "{} player cards: {:#?}, {:#?}",
                faction.faction,
                player_trade_cards.commodity_card_suites(),
                player_trade_cards.calamity_cards()
            );
        }
    }
    check_if_we_can_trade.write(CheckIfWeCanTrade);
}

pub fn transition_to_trade(
    mut check_if_we_can_trade: MessageReader<CheckIfWeCanTrade>,
    players_can_trade_query: Query<(&PlayerTradeCards, Has<IsHuman>)>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in check_if_we_can_trade.read() {
        if players_can_trade_query
            .iter()
            .filter(|(trade, _)| trade.can_trade())
            .count()
            >= 2
        // && players_can_trade_query.iter().filter(|(_, is_human)| *is_human).count() > 0 {
        {
            //debug!("COMMENCE TRADING!");
            next_state.set(GameActivity::Trade);
        } else {
            //debug!("Not enough players can trade");
            next_state.set(GameActivity::PopulationExpansion);
        }
    }
}
