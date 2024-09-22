use bevy::prelude::{default, Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::general::general_enums::GameFaction;

#[derive(Component, Debug, Reflect)]
pub struct GameArea {
    pub id: i32,
}

impl GameArea {
    pub fn new(id: i32) -> Self {
        GameArea { id }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct LandPassage {
    pub to_areas: Vec<Entity>,
}

impl LandPassage {
    pub fn new(to_areas: Vec<Entity>) -> Self {
        LandPassage {
            to_areas,
        }
    }

    pub fn add_passage(&mut self, to_area: Entity) {
        self.to_areas.push(to_area);
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct NeedsConnections {
    pub land_connections: Vec<i32>,
    pub sea_connections: Vec<i32>,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Population {
    pub player_tokens: HashMap<Entity, HashSet<Entity>>,
    pub max_population: usize,
}

impl Population {
    pub fn new(max_population: usize) -> Self {
        Population {
            max_population,
            ..default()
        }
    }

    pub fn remove_tokens(&mut self, player: Entity, tokens: Vec<Entity>) {
        if let Some(player_tokens) = self.player_tokens.get_mut(&player) {
            for token in tokens {
                player_tokens.retain(|t| *t != token);
            }
            if player_tokens.is_empty() {
                self.player_tokens.remove(&player);
            }
        }
    }

    pub fn players(&self) -> HashSet<Entity> {
        self.player_tokens.keys().cloned().collect()
    }
    
    pub fn number_of_players(&self) -> usize {
        self.player_tokens.keys().len()
    }

    pub fn all_lengths_equal(&self) -> bool {
        let first_length = self.player_tokens.values().next().map(|v| v.len());
        self.player_tokens.values().all(|v| Some(v.len()) == first_length)
    }

    pub fn remove_surplus(&mut self) -> HashSet<Entity> {
        assert_eq!(self.number_of_players(), 1); // this should never, ever, not happen
        let surplus_count = self.surplus_count();
        
        let player_tokens = self.player_tokens.values_mut().next().unwrap();
        let tokens: HashSet<Entity> = player_tokens.iter().take(surplus_count).copied().collect();
        
        for token in tokens.iter() {
            player_tokens.remove(token);
        }
        tokens
    }

    pub fn has_surplus(&self) -> bool {
        self.surplus_count() > 0
    }

    pub fn surplus_count(&self) -> usize {
        if self.total_population() > self.max_population {
            self.total_population() - self.max_population
        } else {
            0
        }
    }

    pub fn is_conflict_zone(&self) -> bool {
        self.number_of_players() > 1 && self.has_too_many_tokens()
    }

    pub fn has_too_many_tokens(&self) -> bool {
        self.total_population() > self.max_population
    }

    pub fn total_population(&self) -> usize {
        self.player_tokens.values().map(|set| set.len()).sum()
    }

    pub fn has_population(&self) -> bool {
        self.total_population() > 0
    }
    
    pub fn max_expansion_for_player(&self, player: Entity) -> usize {
        if let Some(player_tokens) = self.player_tokens.get(&player) {
            match player_tokens.len() {
                0 => 0,
                1 => 1,
                _ => 2,
            }
        } else { 0 }
    }

    pub fn population_for_player(&self, player: Entity) -> usize {
        if let Some(player_tokens) = self.player_tokens.get(&player) {
            player_tokens.len()
        } else { 0 }
    }

    pub fn has_player(&self, player: Entity) -> bool {
        self.player_tokens.contains_key(&player)
    }

    pub fn remove_all_but_n_tokens(&mut self, player: Entity, n: usize) -> Option<HashSet<Entity>> {
        let mut tokens_to_remove: usize = 0;
        if let Some(player_tokens) = self.player_tokens.get(&player) {
            tokens_to_remove = if player_tokens.len() > n { player_tokens.len() - n } else { 0 };
        }
        self.remove_tokens_from_area(player, tokens_to_remove)
    }

    pub fn remove_tokens_from_area(&mut self, player: Entity, number_of_tokens: usize) -> Option<HashSet<Entity>> {
        if let Some(player_tokens) = self.player_tokens.get_mut(&player) {
            if number_of_tokens > 0 {
                if player_tokens.len() >= number_of_tokens {
                    let tokens: HashSet<Entity> = player_tokens.iter().take(number_of_tokens).copied().collect();
                    for token in tokens.iter() {
                        player_tokens.remove(token);
                    }
                    if player_tokens.is_empty() { self.player_tokens.remove(&player); }
                    Some(tokens)
                } else {
                    let tokens = player_tokens.drain().collect();
                    self.player_tokens.remove(&player);
                    Some(tokens)
                }
            } else {
                None
            }
        } else { None }
    }

    pub fn add_token_to_area(&mut self, player: Entity, token: Entity) {
        if let Some(tokens) = self.player_tokens.get_mut(&player) {
            tokens.insert(token);
        } else {
            self.player_tokens.insert(player, HashSet::from([token]));
        }
    }
    pub fn remove_token_from_area(&mut self, player: Entity, token: Entity) {
        if let Some(tokens) = self.player_tokens.get_mut(&player) {
            tokens.remove(&token);
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct FloodPlain;

#[derive(Component, Debug, Reflect, Default)]
pub struct Volcano;

#[derive(Component, Debug, Reflect, Default)]
pub struct CityFlood;

#[derive(Component, Debug, Reflect, Default)]
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

impl StartArea {
    pub fn new(faction: GameFaction) -> Self {
        StartArea { faction }
    }
}


#[derive(Component, Debug, Reflect)]
pub struct Faction {
    pub faction: GameFaction,
}

impl Faction {
    pub fn new(faction: GameFaction) -> Self {
        Faction { faction }
    }
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
pub struct PlayerStock {
    pub max_tokens: usize,
    tokens: Vec<Entity>,
}

impl PlayerStock {
    pub fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        PlayerStock {
            max_tokens,
            tokens,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn return_token_to_stock(&mut self, token: Entity) {
        self.tokens.push(token);
    }

    pub fn remove_tokens_from_stock(&mut self, number_of_tokens: usize) -> Option<Vec<Entity>> {
        if self.tokens.len() >= number_of_tokens {
            let tokens = self.tokens.drain(0..number_of_tokens).collect();
            Some(tokens)
        } else {
            None
        }
    }

    pub fn remove_token_from_stock(&mut self) -> Option<Entity> {
        self.tokens.pop()
    }

    pub fn tokens_in_stock(&self) -> usize {
        self.tokens.len()
    }
}


#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerCities {
    pub areas_and_cities: HashMap<Entity, Entity>,
}

impl PlayerCities {
    pub fn has_cities(&self) -> bool {
        !self.has_no_cities()
    }

    pub fn has_no_cities(&self) -> bool {
        self.areas_and_cities.is_empty()
    }

    pub fn number_of_cities(&self) -> usize {
        self.areas_and_cities.len()
    }

    pub fn build_city_in_area(&mut self, area: Entity, city_token: Entity) {
        self.areas_and_cities.insert(area, city_token);
    }

    pub fn remove_city_from_area(&mut self, area: Entity) -> Option<Entity> {
        self.areas_and_cities.remove(&area)
    }

    pub fn has_city_in(&self, area: Entity) -> bool {
        self.areas_and_cities.contains_key(&area)
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerAreas {
    areas: HashSet<Entity>,
    area_population: HashMap<Entity, HashSet<Entity>>,
}

impl PlayerAreas {
    pub fn areas(&self) -> HashSet<Entity> {
        self.areas.clone()
    }

    pub fn areas_and_population(&self) -> HashMap<Entity, HashSet<Entity>> {
        self.area_population.clone()
    }

    pub fn areas_and_population_count(&self) -> HashMap<Entity, usize> {
        self.area_population.clone().iter().map(|(k, v)| (*k, v.len())).collect()
    }

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

    pub fn has_any_population(&self) -> bool {
        !self.areas.is_empty()
    }

    pub fn areas_with_population(&self) -> HashSet<Entity> {
        self.area_population.keys().cloned().collect()
    }

    pub fn total_population(&self) -> usize {
        self.area_population.values().map(|set| set.len()).sum()
    }

    pub fn population_in_area(&self, area: Entity) -> usize {
        if let Some(tokens) = self.area_population.get(&area) {
            tokens.len()
        } else {
            0
        }
    }

    pub fn tokens_for_area(&self, area: Entity) -> Option<HashSet<Entity>> {
        self.area_population.get(&area).cloned()
    }

    pub fn required_tokens_for_expansion(&self) -> usize {
        self.area_population.values().map(|set| {
            match set.len() {
                0 => { 0 }
                1 => { 1 }
                _ => { 2 }
            }
        }).sum()
    }

    pub fn required_tokens_for_expansion_for_area(&self, area: Entity) -> usize {
        if let Some(set) = self.area_population.get(&area) {
            match set.len() {
                0 => { 0 }
                1 => { 1 }
                _ => { 2 }
            }
        } else { 0 }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Treasury {
    tokens: Vec<Entity>,
}

impl Treasury {
    pub fn add_token_to_treasury(&mut self, token: Entity) {
        self.tokens.push(token);
    }

    pub fn remove_token_from_treasury(&mut self) -> Option<Entity> {
        self.tokens.pop()
    }

    pub fn tokens_in_treasury(&self) -> usize {
        self.tokens.len()
    }
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