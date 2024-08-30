use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::HashMap;
use crate::civilization::general::enums::GameFaction;

#[derive(Component, Debug, Reflect)]
pub struct Area;

#[derive(Component, Debug, Reflect, Default)]
pub struct LandPassage {
    pub to_areas: Vec<Entity>,
}

#[derive(Component, Debug, Reflect)]
pub struct NeedsConnections {
    pub land_connections: Vec<String>,
    pub sea_connections: Vec<String>,
}

#[derive(Component, Debug, Reflect)]
pub struct Population {
    pub player_tokens: HashMap<Entity, Vec<Entity>>,
    pub max_population: usize,
    pub total_population: usize,
}

impl Population {
    pub fn new(max_population: usize) -> Self {
        Population {
            player_tokens: HashMap::default(),
            max_population,
            total_population: 0,
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct CitySite;

#[derive(Component, Debug, Reflect)]
pub struct CityToken {
    pub player: Entity,
}

impl CityToken {
    pub fn new(player: Entity) -> Self {
        CityToken { player }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct BuiltCity {
    pub city: Entity,
    pub player: Entity,
}

#[derive(Component, Debug)]
pub struct StartArea {
    pub faction: GameFaction,
}


#[derive(Component, Debug)]
pub struct Faction {
    pub faction: GameFaction,
}

#[derive(Component, Debug, Reflect)]
pub struct Token {
    pub player: Entity,
}

impl Token {
    pub fn new(player: Entity) -> Self {
        Token {
            player
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct Stock {
    pub max_tokens: usize,
    pub tokens: Vec<Entity>,
}

impl Stock {
    pub(crate) fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        Stock {
            max_tokens,
            tokens,
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct Treasury {
    pub tokens: Vec<Entity>,
}

#[derive(Component, Debug, Reflect)]
pub struct CityTokenStock {
    pub max_tokens: usize,
    pub tokens: Vec<Entity>,
}

impl CityTokenStock {
    pub(crate) fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        CityTokenStock {
            max_tokens,
            tokens,
        }
    }
}