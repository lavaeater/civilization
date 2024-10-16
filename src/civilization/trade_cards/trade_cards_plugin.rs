use crate::civilization::general::prelude::*;
use crate::{GameActivity, GameState};
use bevy::app::Startup;
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::prelude::{App, Asset, Commands, FromReflect, OnEnter, Plugin, Query, Reflect, Res, ResMut, Resource, TypePath};
use bevy::utils::{HashMap, HashSet};
use bevy_common_assets::ron::RonAssetPlugin;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

pub struct TradeCardsPlugin;

impl Plugin for TradeCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                RonAssetPlugin::<CivilizationCardDefinitions>::new(&["cards"]))
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let g = asset_server.load("definitions/definitions.cards");
    let cards = CardHandle(g);
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
    pub value: usize,
    pub card_type: TradeCardType,
    pub tradeable: bool,
    pub number: usize,
}

#[derive(Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Reflect)]
pub struct TradeCard {
    pub value: usize,
    pub card_type: TradeCardType,
    pub tradeable: bool,
}

impl TradeCard {
    pub fn new(value: usize, card_type: TradeCardType, tradeable: bool) -> Self {
        Self {
            value,
            card_type,
            tradeable,
        }
    }
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum TradeCardType {
    Commodity(Commodity),
    Calamity(Calamity),
}

#[derive(Clone, Deserialize,
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
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
    Serialize, Debug, Eq, Hash, PartialEq, Reflect)]
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
    IconoclasmAndHeresy,
    Piracy,
}

#[derive(Resource, Debug, Default)]
pub struct CivilizationTradeCards {
    pub card_piles: HashMap<usize, Vec<TradeCard>>,
}

impl CivilizationTradeCards {
    pub fn pull_card_from(&mut self, pile: usize) -> Option<TradeCard> {
        self.card_piles.get_mut(&pile).unwrap().pop()
    }
}

fn acquire_trade_cards(
    player_query: Query<(&PlayerCities, &mut PlayerTradeCards)>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    _commands: Commands,
) {
    for (player_cities, mut player_trade_cards) in player_query
        .iter()
        .sort_by::<&PlayerCities>(|v1, v2| {
            v1.number_of_cities()
                .cmp(&v2.number_of_cities())
        }) {
        (1..player_cities.number_of_cities() +1).into_iter().for_each(|pile| {
            if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                player_trade_cards.cards.push(pulled_card);
            }
            player_trade_cards.cards.push(TradeCard::new(1, TradeCardType::Commodity(Commodity::Ochre), true));
        });
    }
}