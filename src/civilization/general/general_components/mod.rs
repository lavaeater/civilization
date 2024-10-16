use crate::civilization::general::general_enums::GameFaction;
use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::{HashMap, HashSet};
use crate::civilization::trade_cards::prelude::TradeCard;

pub mod population;

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

#[derive(Component, Debug, Reflect)]
pub struct FixTokenPositions;

#[derive(Component, Debug, Reflect)]
pub struct FloodPlain;

#[derive(Component, Debug, Reflect)]
pub struct Volcano;

#[derive(Component, Debug, Reflect)]
pub struct CityFlood;

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

impl BuiltCity {
    pub fn new(city: Entity, player: Entity) -> Self {
        BuiltCity { city, player }
    }
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
    player: Entity,
}

impl Token {
    pub fn new(player: Entity) -> Self {
        Token {
            player
        }
    }
    
    pub fn player(&self) -> Entity {
        self.player
    }
}

#[derive(Component, Debug, Reflect)]
pub struct TokenStock {
    pub max_tokens: usize,
    tokens: HashSet<Entity>,
}

impl TokenStock {
    pub fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        TokenStock {
            max_tokens,
            tokens: HashSet::from_iter(tokens)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn return_token_to_stock(&mut self, token: Entity) {
        self.tokens.insert(token);
    }

    pub fn return_tokens_to_stock(&mut self, tokens: HashSet<Entity>) {
        self.tokens.extend(tokens);
    }

    pub fn remove_tokens_from_stock(&mut self, number_of_tokens: usize) -> Option<HashSet<Entity>> {
        if self.tokens.len() >= number_of_tokens {
            let to_remove: Vec<Entity> = self.tokens.iter().take(number_of_tokens).cloned().collect();
            self.tokens.retain(|t| !to_remove.contains(t));
            Some(to_remove.into_iter().collect())
        } else {
            None
        }
    }

    pub fn remove_at_most_n_tokens_from_stock(&mut self, number_of_tokens: usize) -> Option<HashSet<Entity>> {
        if self.tokens.is_empty() {
            None
        } else if self.tokens.len() >= number_of_tokens {
            self.remove_tokens_from_stock(number_of_tokens)
        } else {
            self.remove_tokens_from_stock(self.tokens.len())
        }
    }

    pub fn remove_token_from_stock(&mut self) -> Option<Entity> {
        // Find an arbitrary item to pop (first item in iteration)
        if let Some(item) = self.tokens.iter().next().cloned() {
            self.tokens.take(&item) // Remove and return the item
        } else {
            None // Return None if the set is empty
        }
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

    pub fn add_tokens_to_area(&mut self, area: Entity, tokens: HashSet<Entity>) {
        self.areas.insert(area);
        if !self.area_population.contains_key(&area) {
            self.area_population.insert(area, HashSet::default());
        }
        self.area_population.get_mut(&area).unwrap().extend(tokens);
    }

    pub fn remove_token_from_area(&mut self, area: &Entity, token: Entity) {
        if let Some(tokens) = self.area_population.get_mut(area) {
            tokens.remove(&token);
            if tokens.is_empty() {
                self.remove_area(area);
            }
        }
    }

    pub fn remove_token(&mut self, token: Entity) {
        let mut key_to_remove: Option<Entity> = None;

        for (area, tokens) in self.area_population.iter_mut() {
            if tokens.remove(&token) {
                if tokens.is_empty() {
                    key_to_remove = Some(*area);
                }
                break; // We can stop after finding the entity
            }
        }

        if let Some(area) = key_to_remove {
            self.remove_area(&area);
        }
    }

    pub fn remove_area(&mut self, area: &Entity) {
        self.areas.remove(area);
        self.area_population.remove(area);
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

    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
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

#[derive(Component, Debug, Reflect, Default)]
pub struct PlayerTradeCards {
    trade_cards: HashSet<TradeCard>,
}