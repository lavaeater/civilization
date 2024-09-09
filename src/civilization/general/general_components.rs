use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::general::general_enums::GameFaction;

#[derive(Component, Debug, Reflect)]
pub struct GameArea;

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
    pub fn remove_tokens_from_area(&mut self, player: Entity, number_of_tokens: usize) -> Option<Vec<Entity>> {
        if let Some(player_tokens) = self.player_tokens.get_mut(&player) {
            if player_tokens.len() >= number_of_tokens {
                let tokens = player_tokens.drain(0..number_of_tokens).collect();
                self.total_population -= number_of_tokens;
                if player_tokens.is_empty() { self.player_tokens.remove(&player); }
                Some(tokens)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add_token_to_area(&mut self, player: Entity, token: Entity) {
        if let Some(tokens) = self.player_tokens.get_mut(&player) {
            tokens.push(token);
        } else {
            self.player_tokens.insert(player, vec![token]);
        }
        self.total_population += 1;
    }

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
    pub fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        Stock {
            max_tokens,
            tokens,
        }
    }
    pub fn remove_tokens_from_stock(&mut self, number_of_tokens: usize) -> Option<Vec<Entity>> {
        if self.tokens.len() >= number_of_tokens {
            let tokens = self.tokens.drain(0..number_of_tokens).collect();
            Some(tokens)
        } else {
            None
        }
    }
}


#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerCities {
    pub areas: HashSet<Entity>,
    pub areas_and_cities: HashMap<Entity, Entity>,
    pub city_tokens: HashSet<Entity>,
}

impl PlayerCities {
    pub fn add_city_to_area(&mut self, area: Entity, city_token: Entity) {
        self.areas.insert(area);
        self.areas_and_cities.insert(area, city_token);
        self.city_tokens.insert(city_token);
    }

    pub fn remove_city_from_area(&mut self, area: Entity) -> Option<Entity> {
        if let Some(city) = self.areas_and_cities.remove(&area) {
            self.city_tokens.remove(&city);
            if self.areas_and_cities.is_empty() {
                self.areas.remove(&area);
            }
            Some(city)
        } else {
            None
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerAreas {
    areas: HashSet<Entity>,
    area_population: HashMap<Entity, HashSet<Entity>>,
}

impl PlayerAreas {
    pub fn add_token_to_area(&mut self, area: Entity, token: Entity) {
        self.areas.insert(area);
        if !self.area_population.contains_key(&area) {
            self.area_population.insert(area, HashSet::default());
        }
        self.area_population.get_mut(&area).unwrap().insert(token);
    }

    pub fn remove_token_from_area(&mut self, area: Entity, token: Entity) {
        if let Some(tokens) = self.area_population.get_mut(&area) {
            tokens.remove(&token);
            if tokens.is_empty() {
                self.area_population.remove(&area);
                self.areas.remove(&area);
            }
        }
    }

    pub fn remove_area(&mut self, area: Entity) {
        self.areas.remove(&area);
        self.area_population.remove(&area);
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Treasury {
    pub tokens: Vec<Entity>,
}

#[derive(Component, Debug, Reflect)]
pub struct CityTokenStock {
    pub max_tokens: usize,
    pub tokens: Vec<Entity>,
}

impl CityTokenStock {
    pub fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        CityTokenStock {
            max_tokens,
            tokens,
        }
    }
}