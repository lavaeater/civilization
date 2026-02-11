use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::civilization::components::*;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::enums::GameFaction;
use crate::civilization::{PlayerTradeCards, Census, TradeCard};
use crate::player::Player;
use crate::stupid_ai::{IsHuman, StupidAi};
use crate::{GameActivity, GameState};

const SAVE_FILE_PATH: &str = "savegame.json";

/// Resource to signal that a game should be loaded
#[derive(Resource)]
pub struct PendingGameLoad(pub GameSaveData);

/// Resource that indicates we're loading from a save file.
/// When present, setup_players should be skipped entirely.
#[derive(Resource)]
pub struct LoadingFromSave;

pub struct SaveGamePlugin;

impl Plugin for SaveGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (save_on_key, trigger_load_on_key))
            .add_systems(
                OnEnter(GameActivity::PrepareGame),
                load_game_from_save.before(crate::civilization::general_systems::setup_players),
            )
            .add_systems(
                OnEnter(GameActivity::StartGame),
                restore_area_populations,
            );
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

fn trigger_load_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
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
                    info!("Parsed save data: round {}, {} players, {} areas",
                        save_data.round, save_data.players.len(), save_data.area_populations.len());
                    // Insert the pending load resource - will be processed on PrepareGame
                    commands.insert_resource(PendingGameLoad(save_data));
                    // Transition to Playing state to trigger the game start
                    next_state.set(GameState::Playing);
                    info!("Loading saved game...");
                }
                Err(e) => error!("Failed to parse save file: {}", e),
            }
        }
        Err(e) => error!("Failed to read save file: {}", e),
    }
}

/// System that loads game state from a pending save file.
/// Runs before setup_players on PrepareGame entry.
fn load_game_from_save(
    mut commands: Commands,
    pending_load: Option<Res<PendingGameLoad>>,
    mut game_info: ResMut<GameInfoAndStuff>,
    _area_query: Query<(Entity, &GameArea, &mut Population)>,
) {
    let Some(pending) = pending_load else {
        return;
    };
    
    // Mark that we're loading from save - this prevents setup_players from running
    commands.insert_resource(LoadingFromSave);
    
    let save_data = &pending.0;
    info!("Loading game from save: round {}, {} players", save_data.round, save_data.players.len());
    
    // Set game round
    game_info.round = save_data.round;
    
    // Create a map of faction -> player entity for later use
    let mut faction_to_player: HashMap<GameFaction, Entity> = HashMap::default();
    
    // Create players
    for (n, saved_player) in save_data.players.iter().enumerate() {
        info!("Creating player: {} ({:?})", saved_player.name, saved_player.faction);
        
        // Create trade cards from saved data
        let trade_cards = PlayerTradeCards::from_cards_vec(saved_player.trade_cards.clone());
        
        // Create player entity
        let player = commands
            .spawn((
                Player,
                Name::new(saved_player.name.clone()),
                Census { population: saved_player.census_population },
                Treasury::default(), // Treasury tokens will be created separately if needed
                Faction::new(saved_player.faction),
                PlayerAreas::default(),
                PlayerCities::default(),
                trade_cards,
            ))
            .id();
        
        // Add AI or Human marker
        if saved_player.is_human {
            commands.entity(player).insert(IsHuman);
            info!("  -> Human player");
        } else {
            commands.entity(player).insert(StupidAi);
        }
        
        // Create tokens for stock
        let tokens: Vec<Entity> = (0..saved_player.tokens_in_stock)
            .map(|_| {
                commands
                    .spawn((Name::new(format!("Token {n}")), Token::new(player)))
                    .id()
            })
            .collect();
        
        // Create city tokens for stock
        let city_tokens: Vec<Entity> = (0..saved_player.city_tokens_in_stock)
            .map(|_| {
                commands
                    .spawn((Name::new(format!("City {n}")), CityToken::new(player)))
                    .id()
            })
            .collect();
        
        commands.entity(player).insert((
            TokenStock::new(47, tokens), // max_tokens is always 47
            CityTokenStock::new(9, city_tokens),
        ));
        
        faction_to_player.insert(saved_player.faction, player);
    }
    
    // Store faction_to_player mapping for area population restoration
    commands.insert_resource(LoadedFactionMap(faction_to_player.clone()));
    commands.insert_resource(PendingAreaPopulations(save_data.area_populations.clone()));
    
    // Remove the pending load resource
    commands.remove_resource::<PendingGameLoad>();
    
    info!("Players created from save. Area populations will be restored after map loads.");
}

/// Resource to map factions to player entities during load
#[derive(Resource)]
pub struct LoadedFactionMap(pub HashMap<GameFaction, Entity>);

/// Resource to hold pending area populations to restore after map loads
#[derive(Resource)]
pub struct PendingAreaPopulations(pub Vec<SavedAreaPopulation>);

/// System to restore area populations from save data.
/// Runs on StartGame entry, after the map has been loaded.
fn restore_area_populations(
    mut commands: Commands,
    pending_pops: Option<Res<PendingAreaPopulations>>,
    faction_map: Option<Res<LoadedFactionMap>>,
    mut area_query: Query<(Entity, &GameArea, &mut Population)>,
    mut player_areas_query: Query<&mut PlayerAreas>,
    mut player_cities_query: Query<&mut PlayerCities>,
    mut city_stock_query: Query<&mut CityTokenStock>,
) {
    let Some(pending) = pending_pops else {
        return;
    };
    let Some(factions) = faction_map else {
        return;
    };
    
    info!("Restoring {} area populations from save", pending.0.len());
    
    // Build a map of area_id -> area_entity
    let area_id_to_entity: HashMap<i32, Entity> = area_query
        .iter()
        .map(|(entity, game_area, _)| (game_area.id, entity))
        .collect();
    
    for saved_area in pending.0.iter() {
        let Some(&area_entity) = area_id_to_entity.get(&saved_area.area_id) else {
            warn!("Area {} not found in map, skipping", saved_area.area_id);
            continue;
        };
        
        // Get mutable population for this area
        let Ok((_, _, mut population)) = area_query.get_mut(area_entity) else {
            continue;
        };
        
        // Add tokens for each faction
        for (faction, token_count) in &saved_area.tokens_by_faction {
            let Some(&player_entity) = factions.0.get(faction) else {
                warn!("Faction {:?} not found in loaded players", faction);
                continue;
            };
            
            // Create tokens and add them to the area
            for _ in 0..*token_count {
                let token = commands
                    .spawn((Name::new("Loaded Token"), Token::new(player_entity)))
                    .id();
                population.add_token_to_area(player_entity, token);
                
                // Update player areas
                if let Ok(mut player_areas) = player_areas_query.get_mut(player_entity) {
                    player_areas.add_token_to_area(area_entity, token);
                }
            }
            
            info!("  Area {}: {} tokens for {:?}", saved_area.area_id, token_count, faction);
        }
        
        // Build city if needed
        if let Some(city_faction) = &saved_area.city_owner {
            let Some(&player_entity) = factions.0.get(city_faction) else {
                warn!("City owner faction {:?} not found", city_faction);
                continue;
            };
            
            // Get a city token from the player's stock
            if let Ok(mut city_stock) = city_stock_query.get_mut(player_entity) {
                if let Some(city_token) = city_stock.get_token_from_stock() {
                    // Add BuiltCity component to the area
                    commands.entity(area_entity).insert(BuiltCity::new(city_token, player_entity));
                    
                    // Update player cities
                    if let Ok(mut player_cities) = player_cities_query.get_mut(player_entity) {
                        player_cities.build_city_in_area(area_entity, city_token);
                    }
                    
                    info!("  Area {}: city built by {:?}", saved_area.area_id, city_faction);
                } else {
                    warn!("No city tokens available for {:?}", city_faction);
                }
            }
        }
    }
    
    // Clean up resources
    commands.remove_resource::<PendingAreaPopulations>();
    commands.remove_resource::<LoadedFactionMap>();
    commands.remove_resource::<LoadingFromSave>();
    
    info!("Area populations restored from save");
}
