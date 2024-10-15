use bevy::prelude::{App, Commands, Component, OnEnter, Plugin, Query, Resource};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::census::prelude::GameInfoAndStuff;
use crate::civilization::general::prelude::*;
use crate::GameActivity;

pub struct TradeCardsPlugin;

impl Plugin for TradeCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                 OnEnter(GameActivity::AcquireTradeCards), acquire_trade_cards)
        ;
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TradeCard;

#[derive(Resource, Debug)]
pub struct TradeCards {
    pub card_piles: HashMap<usize, HashSet<TradeCard>>
}

impl Default for TradeCards {
    fn default() -> Self {
        Self {
            card_piles: HashMap::from([(1, HashSet::from([])),(2)])
        }
    }
}

fn acquire_trade_cards(
    player_query: Query<&PlayerCities>,
    mut commands: Commands,
) {
    for player_cities in player_query.iter().sort_by::<&PlayerCities>(|v1, v2| v1.number_of_cities().cmp(&v2.number_of_cities())) {
        
    }
}