use crate::civilization::general::prelude::*;
use crate::{GameActivity, GameState};
use bevy::app::Startup;
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::prelude::{App, Asset, Commands, Component, OnEnter, Plugin, Query, Reflect, Res, ResMut, Resource, Transform, TypePath};
use bevy::utils::{HashMap, HashSet};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};

pub struct TradeCardsPlugin;

impl Plugin for TradeCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                RonAssetPlugin::<CivilizationCardDefinitions>::new(&["cards.ron"]))
            .add_systems(Startup, setup)
            .add_systems(OnEnter(GameState::Playing), load_civilization_cards)
            .add_systems(
                OnEnter(GameActivity::AcquireTradeCards), acquire_trade_cards)
        ;
    }
}

fn load_civilization_cards(
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
            card_resource.card_piles.entry(trading_card.value).or_insert(HashSet::new()).extend(cards);
        }
        commands.insert_resource(card_resource);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cards = CardHandle(asset_server.load("definitions/definitions.cards.ron"));
    commands.insert_resource(cards);
}

#[derive(Deserialize, Serialize, Asset, TypePath, Clone)]
pub struct CivilizationCardDefinitions {
    pub trading_cards: Vec<TradeCardDefinition>,
}

#[derive(Resource)]
struct CardHandle(Handle<CivilizationCardDefinitions>);

#[derive(
    Deserialize,
    Serialize,
    Asset,
    TypePath,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
)]
pub struct TradeCardDefinition {
    pub value: u8,
    pub card_type: TradeCardType,
    pub tradeable: bool,
    pub number: usize,
}

#[derive(Clone,
    Debug,
    Eq,
    PartialEq,
    Hash)]
pub struct TradeCard {
    pub value: u8,
    pub card_type: TradeCardType,
    pub tradeable: bool,
}

impl TradeCard {
    pub fn new(value: u8, card_type: TradeCardType, tradeable: bool) -> Self {
        Self {
            value,
            card_type,
            tradeable,
        }
    }
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq)]
pub enum TradeCardType {
    Commodity(Commodity),
    Calamity(Calamity),
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq)]
pub enum Commodity {
    Ochre,
    Hides,
    Iron,
    Papyrus,
    Salt,
    Timber,
    Grain,
    Oil,
    Cloth,
    Wine,
    Bronze,
    Silver,
    Spices,
    Resin,
    Gems,
    Dye,
    Gold,
    Ivory,
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq)]
pub enum Calamity {
    VolcanoEarthquake,
    Treachery,
    Famine,
    Superstition,
    CivilWar,
    SlaveRevolt,
    Flood,
    BarbarianHordes,
    Epidemic,
    CivilDisorder,
    IcononoclasmAndHeresy,
    Piracy,
}

#[derive(Resource, Debug, Default)]
pub struct CivilizationTradeCards {
    pub card_piles: HashMap<u8, HashSet<TradeCard>>,
}

fn acquire_trade_cards(
    player_query: Query<&PlayerCities>,
    mut commands: Commands,
) {
    for player_cities in player_query.iter().sort_by::<&PlayerCities>(|v1, v2| v1.number_of_cities().cmp(&v2.number_of_cities())) {}
}