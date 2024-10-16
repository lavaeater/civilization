use bevy::prelude::{Commands, NextState, Query, Res, ResMut, Resource};
use bevy::asset::{AssetServer, Assets, Handle};
use rand::prelude::SliceRandom;
use crate::civilization::general::general_components::{PlayerCities, PlayerTradeCards, TradeCard};
use crate::civilization::trade_cards::trade_card_components::CivilizationCardDefinitions;
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
        (1..player_cities.number_of_cities() +1).into_iter().for_each(|pile| {
            if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                player_trade_cards.add_trade_card(pulled_card);
            }
        });
    }
    next_state.set(GameActivity::PopulationExpansion);
}

pub fn load_civilization_cards(
    mut commands: Commands,
    card_handle: Res<CardHandle>,
    cards: Res<Assets<CivilizationCardDefinitions>>,
) {
    if let Some(civ_cards) = cards.get(card_handle.0.id()) {
        let mut card_resource = CivilizationTradeCards::default();
        for trading_card in civ_cards.trading_cards.iter() {
            let cards = (0..trading_card.number).map(|_| {
                TradeCard::new(trading_card.value, trading_card.card_type.clone(), trading_card.tradeable)
            }).collect::<Vec<_>>();
            card_resource.card_piles.entry(trading_card.value).or_insert(vec![]).extend(cards);
        }
        // needs some more sophistication, there should be at least player count number of cards
        // before a calamity shows up.
        for pile in card_resource.card_piles.values_mut() {
            pile.shuffle(&mut rand::thread_rng());
        }
        commands.insert_resource(card_resource);
    }
}

#[derive(Resource)]
pub struct CardHandle(Handle<CivilizationCardDefinitions>);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let g = asset_server.load("definitions/definitions.cards");
    let cards = CardHandle(g);
    commands.insert_resource(cards);
}