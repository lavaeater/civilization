use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::HashMap;

#[derive(Component, Debug, Reflect)]
pub struct Area {
    pub max_population: u8,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct LandPassage {
    pub to_areas: Vec<Entity>,
}

#[derive(Component, Debug, Reflect)]
pub struct NeedsConnections {
    pub land_connections: Vec<String>,
    pub sea_connections: Vec<String>,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Population {
    pub player_tokens: HashMap<Entity, Vec<Entity>>,
    pub total_population: usize
}

#[derive(Component, Debug)]
pub struct StartArea;

#[derive(Component, Debug, Reflect)]
pub struct Token {
    pub player: Entity
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