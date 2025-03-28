use crate::civilization::concepts::trade_cards::components::{CivilizationCardDefinitions, CivilizationTradeCards, TradeCard};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::prelude::{Commands, Res, Resource};
use rand::prelude::SliceRandom;

pub fn load_civilization_cards(
    mut commands: Commands,
    card_handle: Res<CardHandle>,
    cards: Res<Assets<CivilizationCardDefinitions>>,
) {
    if let Some(civ_cards) = cards.get(card_handle.0.id()) {
        let mut card_resource = CivilizationTradeCards::default();
        for trading_card in civ_cards.trading_cards.iter() {
            let cards = (0..trading_card.number).map(|_| {
                TradeCard::from_def(trading_card)
            }).collect::<Vec<_>>();
            card_resource.card_piles.entry(trading_card.value).or_insert(vec![]).extend(cards);
        }
        // needs some more sophistication, there should be at least player count number of cards
        // before a calamity shows up.
        for pile in card_resource.card_piles.values_mut() {
            pile.shuffle(&mut rand::rng());
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