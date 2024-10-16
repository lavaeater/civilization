use crate::civilization::census::prelude::GameInfoAndStuff;
use crate::civilization::general::prelude::*;
use crate::GameActivity;
use bevy::prelude::{App, Commands, Component, OnEnter, Plugin, Query, Resource};
use bevy::utils::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

pub struct TradeCardsPlugin;

impl Plugin for TradeCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::AcquireTradeCards), acquire_trade_cards)
        ;
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
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
    pub number: u8
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
    Ivory
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
    Piracy
}

#[derive(Resource, Debug)]
pub struct TradeCards {
    pub card_piles: HashMap<u8, HashSet<TradeCard>>,
}

impl Default for TradeCards {
    fn default() -> Self {
        Self {
            card_piles: HashMap::from([(1, HashSet::from([])), (2, HashSet::from([]))])
        }
    }
}

fn acquire_trade_cards(
    player_query: Query<&PlayerCities>,
    mut commands: Commands,
) {
    for player_cities in player_query.iter().sort_by::<&PlayerCities>(|v1, v2| v1.number_of_cities().cmp(&v2.number_of_cities())) {}
}