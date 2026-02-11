use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::civilization::components::*;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::enums::GameFaction;
use crate::civilization::{PlayerTradeCards, Census, TradeCard};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;

const SAVE_FILE_PATH: &str = "savegame.json";

pub struct SaveGamePlugin;

impl Plugin for SaveGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (save_on_key, load_on_key));
    }
}

/// Saved data for a single player
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedPlayer {
    pub name: String,
    pub faction: GameFaction,
    pub is_human: bool,
    pub census_population: usize,
    pub treasury: usize,
    pub tokens_in_stock: usize,
    pub city_tokens_in_stock: usize,
    pub trade_cards: Vec<(TradeCard, usize)>,
}

/// Saved data for population in an area
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedAreaPopulation {
    pub area_id: i32,
    /// Map of faction -> number of tokens
    pub tokens_by_faction: Vec<(GameFaction, usize)>,
    /// Faction that has a city here, if any
    pub city_owner: Option<GameFaction>,
}

/// Complete game save data
#[derive(Serialize, Deserialize, Debug)]
pub struct GameSaveData {
    pub round: usize,
    pub game_activity: String,
    pub players: Vec<SavedPlayer>,
    pub area_populations: Vec<SavedAreaPopulation>,
}

fn save_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    game_info: Res<GameInfoAndStuff>,
    current_activity: Option<Res<State<GameActivity>>>,
    player_query: Query<(
        &Name,
        &Faction,
        &Census,
        &Treasury,
        &TokenStock,
        &CityTokenStock,
        &PlayerTradeCards,
        Has<IsHuman>,
    ), With<Player>>,
    area_query: Query<(&GameArea, &Population, Option<&BuiltCity>)>,
    faction_query: Query<&Faction>,
) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }
    
    info!("Saving game...");
    
    // Collect player data
    let mut players = Vec::new();
    for (name, faction, census, treasury, token_stock, city_stock, trade_cards, is_human) in player_query.iter() {
        let saved_player = SavedPlayer {
            name: name.to_string(),
            faction: faction.faction,
            is_human,
            census_population: census.population,
            treasury: treasury.tokens_in_treasury(),
            tokens_in_stock: token_stock.tokens_in_stock(),
            city_tokens_in_stock: city_stock.city_tokens_in_stock(),
            trade_cards: trade_cards.cards_as_vec(),
        };
        players.push(saved_player);
    }
    
    // Collect area population data
    let mut area_populations = Vec::new();
    for (game_area, population, built_city) in area_query.iter() {
        let mut tokens_by_faction = Vec::new();
        for (player_entity, tokens) in population.player_tokens().iter() {
            if let Ok(faction) = faction_query.get(*player_entity) {
                tokens_by_faction.push((faction.faction, tokens.len()));
            }
        }
        
        let city_owner = built_city.and_then(|bc| {
            faction_query.get(bc.player).ok().map(|f| f.faction)
        });
        
        if !tokens_by_faction.is_empty() || city_owner.is_some() {
            area_populations.push(SavedAreaPopulation {
                area_id: game_area.id,
                tokens_by_faction,
                city_owner,
            });
        }
    }
    
    let activity_name = current_activity
        .map(|a| format!("{:?}", a.get()))
        .unwrap_or_else(|| "Unknown".to_string());
    
    let save_data = GameSaveData {
        round: game_info.round,
        game_activity: activity_name,
        players,
        area_populations,
    };
    
    match serde_json::to_string_pretty(&save_data) {
        Ok(json) => {
            if let Err(e) = fs::write(SAVE_FILE_PATH, &json) {
                error!("Failed to write save file: {}", e);
            } else {
                info!("Game saved to {} ({} players, {} areas with population)", 
                    SAVE_FILE_PATH, save_data.players.len(), save_data.area_populations.len());
            }
        }
        Err(e) => error!("Failed to serialize save data: {}", e),
    }
}

fn load_on_key(
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::F9) {
        return;
    }
    
    info!("Loading game...");
    
    if !Path::new(SAVE_FILE_PATH).exists() {
        warn!("No save file found at {}", SAVE_FILE_PATH);
        return;
    }
    
    match fs::read_to_string(SAVE_FILE_PATH) {
        Ok(json) => {
            match serde_json::from_str::<GameSaveData>(&json) {
                Ok(save_data) => {
                    info!("Loaded save data: round {}, {} players, {} areas",
                        save_data.round, save_data.players.len(), save_data.area_populations.len());
                    // TODO: Implement actual game state restoration
                    // This will require triggering a state transition and spawning entities
                }
                Err(e) => error!("Failed to parse save file: {}", e),
            }
        }
        Err(e) => error!("Failed to read save file: {}", e),
    }
}
