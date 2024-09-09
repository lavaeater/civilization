use bevy::prelude::{default, Component, Entity, Reflect};
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use bevy::utils::{HashMap, HashSet};
use itertools::Itertools;
use crate::civilization::general::general_enums::GameFaction;

#[derive(Component, Debug, Reflect, Default)]
pub struct GameArea;

#[derive(Component, Debug, Reflect, Default)]
pub struct LandPassage {
    pub to_areas: Vec<Entity>,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct NeedsConnections {
    pub land_connections: Vec<String>,
    pub sea_connections: Vec<String>,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Population {
    pub player_tokens: HashMap<Entity, Vec<Entity>>,
    pub max_population: usize,
}

impl Population {
    pub fn new(max_population: usize) -> Self {
        Population {
            max_population,
            ..default()
        }
    }
    
    pub fn remove_surplus(&mut self) {
        
    }
    
    pub fn has_surplus(&self)-> bool {
        self.surplus_count() > 0
    }
    
    pub fn surplus_count(&self) -> usize {
        let surplus: i32 = self.total_population().try_into().unwrap_or(0) - self.max_population.try_into().unwrap_or(0);
        if surplus > 0 {
            surplus.into()
        } else { 0 } 
    }
    
    pub fn is_conflict_zone(&self) -> bool {
        self.number_of_players() > 1 && self.has_too_many_tokens()
    }
    
    pub fn has_too_many_tokens(&self) -> bool {
        self.total_population() > self.max_population
    }
    
    pub fn total_population(&self)-> usize {
        self.player_tokens.values().map(|set| set.len()).sum()
    }

    pub fn has_population(&self)-> bool {
        self.total_population() > 0
    }
    
    pub fn number_of_players(&self) -> usize {
        self.player_tokens.keys().len()
    }

    pub fn remove_tokens_from_area(&mut self, player: Entity, number_of_tokens: usize) -> Option<Vec<Entity>> {
        if let Some(player_tokens) = self.player_tokens.get_mut(&player) {
            if player_tokens.len() >= number_of_tokens {
                let mut tokens = player_tokens.drain(0..number_of_tokens).collect();
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
            self.player_tokens.insert(player,  vec![token]);
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
    pub fn contains(&self, area: Entity) -> bool {
        self.areas.contains(&area) && self.area_population.contains_key(&area) && !self.area_population.get(&area).unwrap().is_empty()
    }

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
                self.remove_area(area);
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
    tokens: Vec<Entity>,
}

impl CityTokenStock {
    pub fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        CityTokenStock {
            max_tokens,
            tokens,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn get_token_from_stock(&mut self) -> Option<Entity> {
        self.tokens.pop()
    }

    pub fn return_token_to_stock(&mut self, token: Entity) {
        self.tokens.push(token);
    }
}