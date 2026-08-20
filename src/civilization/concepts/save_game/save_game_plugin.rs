use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::civilization::components::*;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::city_construction::{CityConstructionPhaseActive, IsBuilding};
use crate::civilization::concepts::movement::movement_components::PerformingMovement;
use crate::civilization::concepts::population_expansion::population_expansion_components::NeedsExpansion;
use crate::civilization::concepts::ships::create_ship_stock;
use crate::civilization::concepts::AvailableFactions;
use crate::civilization::enums::GameFaction;
use crate::civilization::game_moves::RecalculatePlayerMoves;
use crate::civilization::{AstPosition, CanTrade, CivCardName, PlayerAcquiringCivilizationCards, PlayerCivilizationCards, PlayerTradeCards, Census, TradeCard};
use crate::player::Player;
use crate::stupid_ai::{IsHuman, Personality, Playstyle, StupidAi};
use crate::{GameActivity, GameState};

const SAVE_FILE_PATH: &str = "savegame.json";
const SAVE_GAME_VERSION: &str = "0.0.2";

/// Dev-workflow env var: when set (to anything), the game skips straight
/// past the menu and loads `savegame.json` on boot if it exists -- lets you
/// iterate on UI/logic, restart, and land right back where you left off
/// instead of re-driving the menu and early turns by hand every time.
const AUTOLOAD_ENV_VAR: &str = "ADV_CIV_AUTOLOAD";

/// Message to request a game save (fired by F5 key or menu button)
#[derive(Message)]
pub struct SaveGameRequest;

/// Message to request a game load (fired by F9 key or menu button)
#[derive(Message)]
pub struct LoadGameRequest;

/// Resource to signal that a game should be loaded
#[derive(Resource)]
pub struct PendingGameLoad(pub GameSaveData);

/// Resource that indicates we're loading from a save file.
/// When present, setup_players should be skipped entirely.
/// Also carries the saved activity and per-player completion state
/// so that OnEnter systems can skip already-completed players.
#[derive(Resource)]
pub struct LoadingFromSave {
    /// The activity the game was in when saved
    pub saved_activity: GameActivity,
    /// Factions that had already completed the current activity when saved
    pub completed_factions: Vec<GameFaction>,
    /// Saved census order (as factions, resolved to entities after load)
    pub census_order: Vec<GameFaction>,
    /// Saved left_to_move (as factions, resolved to entities after load)
    pub left_to_move: Vec<GameFaction>,
    /// The faction that was actively moving when saved (already popped from left_to_move)
    pub current_mover: Option<GameFaction>,
}

pub struct SaveGamePlugin;

impl Plugin for SaveGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<SaveGameRequest>()
            .add_message::<LoadGameRequest>()
            .add_systems(
                Update,
                (
                    save_on_key,
                    trigger_load_on_key,
                    autosave_on_player_move,
                    handle_save_request,
                    handle_load_request,
                ),
            )
            .add_systems(OnEnter(GameState::Menu), trigger_autoload)
            .add_systems(
                OnEnter(GameActivity::PrepareGame),
                load_game_from_save.before(crate::civilization::general_systems::setup_players),
            )
            .add_systems(
                OnEnter(GameActivity::StartGame),
                restore_area_populations,
            )
            // Safety net: clean up LoadingFromSave for atomic activities that don't
            // consume it themselves (Census, Conflict, RemoveSurplus, CheckCitySupport,
            // AcquireTradeCards, Trade). The per-player activities (PopExpansion,
            // Movement, CityConstruction) clean it up in their own OnEnter systems.
            .add_systems(OnEnter(GameActivity::Census), cleanup_loading_from_save)
            .add_systems(OnEnter(GameActivity::Conflict), cleanup_loading_from_save)
            .add_systems(OnEnter(GameActivity::RemoveSurplusPopulation), cleanup_loading_from_save)
            .add_systems(OnEnter(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation), cleanup_loading_from_save)
            .add_systems(OnEnter(GameActivity::AcquireTradeCards), cleanup_loading_from_save)
            .add_systems(OnEnter(GameActivity::Trade), cleanup_loading_from_save);
    }
}

fn cleanup_loading_from_save(
    mut commands: Commands,
    loading_from_save: Option<Res<LoadingFromSave>>,
) {
    if loading_from_save.is_some() {
        info!("Cleaning up LoadingFromSave resource (atomic activity)");
        commands.remove_resource::<LoadingFromSave>();
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
    /// Whether this player has completed the current game activity.
    /// Used on load to avoid re-running activity logic for players who already finished.
    #[serde(default)]
    pub done_with_current_activity: bool,
    #[serde(default = "default_ast_space")]
    pub ast_space: u32,
    /// Civilization cards owned at save time (rule 31.x). Older save files
    /// predate this field; `#[serde(default)]` loads them as owning none
    /// rather than rejecting the whole file.
    #[serde(default)]
    pub owned_civ_cards: Vec<CivCardName>,
}

fn default_ast_space() -> u32 { 0 }

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
    pub version: String,
    pub round: usize,
    pub game_activity: GameActivity,
    pub players: Vec<SavedPlayer>,
    pub area_populations: Vec<SavedAreaPopulation>,
    /// Census order saved as factions (resolved to entities on load)
    #[serde(default)]
    pub census_order: Vec<GameFaction>,
    /// Players left to move, saved as factions (resolved to entities on load)
    #[serde(default)]
    pub left_to_move: Vec<GameFaction>,
    /// The faction currently performing movement (already popped from left_to_move)
    #[serde(default)]
    pub current_mover: Option<GameFaction>,
}

/// Determine whether a player has completed the current game activity.
/// A player is "done" if they no longer have the marker component for that activity.
fn is_player_done_with_activity(
    player: Entity,
    activity: &GameActivity,
    needs_expansion_query: &Query<Entity, With<NeedsExpansion>>,
    performing_movement_query: &Query<Entity, With<PerformingMovement>>,
    is_building_query: &Query<Entity, With<IsBuilding>>,
    left_to_move: &[Entity],
    can_trade_query: &Query<Entity, With<CanTrade>>,
    acquiring_civ_cards_query: &Query<Entity, With<PlayerAcquiringCivilizationCards>>,
) -> bool {
    match activity {
        GameActivity::PopulationExpansion => {
            // Player is done if they no longer have NeedsExpansion
            needs_expansion_query.get(player).is_err()
        }
        GameActivity::Movement => {
            // Player is done if they're not currently performing movement
            // AND they're not in the left_to_move list
            performing_movement_query.get(player).is_err()
                && !left_to_move.contains(&player)
        }
        GameActivity::CityConstruction => {
            // Player is done if they no longer have IsBuilding
            is_building_query.get(player).is_err()
        }
        GameActivity::Trade => {
            // Player is done trading once CanTrade is removed from them.
            can_trade_query.get(player).is_err()
        }
        GameActivity::AcquireCivilizationCards => {
            // Player is done once PlayerAcquiringCivilizationCards is removed.
            acquiring_civ_cards_query.get(player).is_err()
        }
        // Atomic activities (Census, Conflict, RemoveSurplus, etc.) run in one shot,
        // so if we're saved during them, no player has partial state.
        _ => false,
    }
}

/// Autosave for UI iteration: `RecalculatePlayerMoves` fires after
/// essentially every player action across every phase (movement, city
/// construction, trade, civ card purchase, population expansion, plus the
/// generic per-phase "needs a move" triggers), so it's the closest thing this
/// codebase has to a single "a move happened" choke point. Firing a save
/// after each batch means `savegame.json` always reflects roughly the last
/// move made, so a UI bug can be reached again with F9 or `ADV_CIV_AUTOLOAD`
/// instead of replaying the whole game by hand.
fn autosave_on_player_move(
    mut recalc_reader: MessageReader<RecalculatePlayerMoves>,
    mut save_writer: MessageWriter<SaveGameRequest>,
) {
    if recalc_reader.read().next().is_some() {
        recalc_reader.clear();
        save_writer.write(SaveGameRequest);
    }
}

/// Dev-workflow convenience: skip the menu and jump straight into the last
/// autosaved game when `ADV_CIV_AUTOLOAD` is set, so restarting the app to
/// pick up a code/UI change lands back where you left off.
fn trigger_autoload(mut writer: MessageWriter<LoadGameRequest>) {
    if std::env::var(AUTOLOAD_ENV_VAR).is_ok() && Path::new(SAVE_FILE_PATH).exists() {
        info!("{} set -- autoloading {}", AUTOLOAD_ENV_VAR, SAVE_FILE_PATH);
        writer.write(LoadGameRequest);
    }
}

fn save_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SaveGameRequest>,
) {
    if keys.just_pressed(KeyCode::F5) {
        writer.write(SaveGameRequest);
    }
}

fn handle_save_request(
    mut events: MessageReader<SaveGameRequest>,
    game_info: Res<GameInfoAndStuff>,
    current_activity: Option<Res<State<GameActivity>>>,
    player_query: Query<(
        Entity,
        &Name,
        &Faction,
        &Census,
        &Treasury,
        &TokenStock,
        &CityTokenStock,
        &PlayerTradeCards,
        Has<IsHuman>,
        Option<&AstPosition>,
        Option<&PlayerCivilizationCards>,
    ), With<Player>>,
    area_query: Query<(&GameArea, &Population, Option<&BuiltCity>)>,
    faction_query: Query<&Faction>,
    needs_expansion_query: Query<Entity, With<NeedsExpansion>>,
    performing_movement_query: Query<Entity, With<PerformingMovement>>,
    is_building_query: Query<Entity, With<IsBuilding>>,
    can_trade_query: Query<Entity, With<CanTrade>>,
    acquiring_civ_cards_query: Query<Entity, With<PlayerAcquiringCivilizationCards>>,
) {
    if events.read().next().is_none() {
        return;
    }
    
    let activity = current_activity
        .as_ref()
        .map(|a| a.get().clone())
        .unwrap_or_default();
    
    info!("Saving game (activity: {:?})...", activity);
    
    // Collect player data with per-player completion state
    let mut players = Vec::new();
    for (entity, name, faction, census, treasury, token_stock, city_stock, trade_cards, is_human, ast_pos, civ_cards) in player_query.iter() {
        let done = is_player_done_with_activity(
            entity,
            &activity,
            &needs_expansion_query,
            &performing_movement_query,
            &is_building_query,
            &game_info.left_to_move,
            &can_trade_query,
            &acquiring_civ_cards_query,
        );
        let saved_player = SavedPlayer {
            name: name.to_string(),
            faction: faction.faction,
            is_human,
            census_population: census.population,
            treasury: treasury.tokens_in_treasury(),
            tokens_in_stock: token_stock.tokens_in_stock(),
            city_tokens_in_stock: city_stock.city_tokens_in_stock(),
            trade_cards: trade_cards.cards_as_vec(),
            done_with_current_activity: done,
            ast_space: ast_pos.map_or(0, |p| p.space),
            owned_civ_cards: civ_cards.map_or_else(Vec::new, |c| c.cards.iter().copied().collect()),
        };
        if done {
            info!("  Player {} ({:?}) is DONE with {:?}", name, faction.faction, activity);
        } else {
            info!("  Player {} ({:?}) is NOT done with {:?}", name, faction.faction, activity);
        }
        players.push(saved_player);
    }
    
    // Collect area population data
    let mut area_populations = Vec::new();
    for (game_area, population, built_city) in area_query.iter() {
        let mut tokens_by_faction = Vec::new();
        for (player_entity, tokens) in population.player_tokens() {
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
    
    // Save census_order and left_to_move as faction lists
    let census_order: Vec<GameFaction> = game_info.census_order.iter()
        .filter_map(|e| faction_query.get(*e).ok().map(|f| f.faction))
        .collect();
    let left_to_move: Vec<GameFaction> = game_info.left_to_move.iter()
        .filter_map(|e| faction_query.get(*e).ok().map(|f| f.faction))
        .collect();
    
    // Find the player currently performing movement (already popped from left_to_move)
    let current_mover = if activity == GameActivity::Movement {
        performing_movement_query.iter().next().and_then(|e| {
            faction_query.get(e).ok().map(|f| f.faction)
        })
    } else {
        None
    };
    
    let save_data = GameSaveData {
        version: SAVE_GAME_VERSION.to_string(),
        round: game_info.round,
        game_activity: activity,
        players,
        area_populations,
        census_order,
        left_to_move,
        current_mover,
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
    mut writer: MessageWriter<LoadGameRequest>,
) {
    if keys.just_pressed(KeyCode::F9) {
        writer.write(LoadGameRequest);
    }
}

fn handle_load_request(
    mut events: MessageReader<LoadGameRequest>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if events.read().next().is_none() {
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
                    if save_data.version != SAVE_GAME_VERSION {
                        error!("Save file version mismatch: expected {}, got {}. Save file rejected.",
                            SAVE_GAME_VERSION, save_data.version);
                        return;
                    }
                    info!("Parsed save data (v{}): round {}, {} players, {} areas",
                        save_data.version, save_data.round, save_data.players.len(), save_data.area_populations.len());
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
    
    let save_data = &pending.0;
    info!("Loading game from save: round {}, {} players", save_data.round, save_data.players.len());
    
    // Determine which factions have completed the current activity
    let completed_factions: Vec<GameFaction> = save_data.players.iter()
        .filter(|p| p.done_with_current_activity)
        .map(|p| p.faction)
        .collect();
    
    // Mark that we're loading from save - this prevents setup_players from running
    // and carries activity state for OnEnter systems
    commands.insert_resource(LoadingFromSave {
        saved_activity: save_data.game_activity.clone(),
        completed_factions,
        census_order: save_data.census_order.clone(),
        left_to_move: save_data.left_to_move.clone(),
        current_mover: save_data.current_mover,
    });
    
    // Set game round
    game_info.round = save_data.round;
    
    // Create a map of faction -> player entity for later use
    let mut faction_to_player: HashMap<GameFaction, Entity> = HashMap::default();
    
    // Create players
    for (n, saved_player) in save_data.players.iter().enumerate() {
        info!("Creating player: {} ({:?})", saved_player.name, saved_player.faction);
        
        // Create trade cards from saved data
        let trade_cards = PlayerTradeCards::from_cards_vec(saved_player.trade_cards.clone());
        let civ_cards = PlayerCivilizationCards {
            cards: saved_player.owned_civ_cards.iter().copied().collect(),
        };

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
                civ_cards,
                AstPosition::new(saved_player.ast_space),
            ))
            .id();
        
        // Add AI or Human marker
        if saved_player.is_human {
            commands.entity(player).insert(IsHuman);
            info!("  -> Human player");
        } else {
            // Every AI move-selection system (select_stupid_movement et al.)
            // requires `&Personality` non-optionally and, being event-driven
            // rather than requeryable, silently drops the SelectStupidMove
            // event with no retry when the query fails -- an AI player
            // missing this component doesn't error, it just never moves
            // again, hanging that phase forever. setup_players round-robins
            // over every archetype for a fresh game; do the same here so a
            // loaded game's AI table looks the same as it would have fresh.
            let playstyle = Playstyle::ALL[n % Playstyle::ALL.len()];
            commands
                .entity(player)
                .insert((StupidAi, Personality::from_playstyle(playstyle)));
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
        
        // Ship stock/board state isn't part of the save format yet (rule 22.4's
        // 4-ship cap is a fresh default here, same as a new game) -- without
        // this, loaded players were missing ShipStock/PlayerShips entirely,
        // which silently broke every query that requires them (movement-move
        // generation, the Player Info HUD's "current human" lookup) rather
        // than just losing ship state gracefully.
        let (ship_stock, player_ships) = create_ship_stock(&mut commands, player);

        commands.entity(player).insert((
            TokenStock::new(47, tokens), // max_tokens is always 47
            CityTokenStock::new(9, city_tokens),
            ship_stock,
            player_ships,
        ));

        faction_to_player.insert(saved_player.faction, player);
    }
    
    // Store faction_to_player mapping for area population restoration
    commands.insert_resource(LoadedFactionMap(faction_to_player.clone()));
    commands.insert_resource(PendingAreaPopulations(save_data.area_populations.clone()));
    
    // Insert activity-specific resources based on saved activity
    if save_data.game_activity == GameActivity::CityConstruction {
        commands.insert_resource(CityConstructionPhaseActive);
        info!("Inserted CityConstructionPhaseActive resource for loaded save");
    }
    
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
    mut area_query: Query<(Entity, &GameArea, &mut Population, &Transform)>,
    mut player_areas_query: Query<&mut PlayerAreas>,
    mut player_cities_query: Query<(Entity, &mut PlayerCities, &mut CityTokenStock)>,
    game_factions: Res<AvailableFactions>,
) {
    let Some(pending) = pending_pops else {
        return;
    };
    let Some(factions) = faction_map else {
        return;
    };
    
    info!("Restoring {} area populations from save", pending.0.len());
    
    // Build a map of area_id -> (area_entity, transform)
    let area_id_to_entity: HashMap<i32, (Entity, Vec3)> = area_query
        .iter()
        .map(|(entity, game_area, _, transform)| (game_area.id, (entity, transform.translation)))
        .collect();
    
    for saved_area in &pending.0 {
        let Some(&(area_entity, area_position)) = area_id_to_entity.get(&saved_area.area_id) else {
            warn!("Area {} not found in map, skipping", saved_area.area_id);
            continue;
        };
        
        // Get mutable population for this area
        let Ok((_, _, mut population, _)) = area_query.get_mut(area_entity) else {
            continue;
        };
        
        // Add tokens for each faction
        for (faction, token_count) in &saved_area.tokens_by_faction {
            let Some(&player_entity) = factions.0.get(faction) else {
                warn!("Faction {:?} not found in loaded players", faction);
                continue;
            };
            
            // Get the faction icon for this faction
            let Some(faction_icon) = game_factions.faction_icons.get(faction) else {
                warn!("No icon for faction {:?}", faction);
                continue;
            };
            
            // Create tokens and add them to the area
            for _ in 0..*token_count {
                let token = commands
                    .spawn((
                        Name::new("Loaded Token"),
                        Token::new(player_entity),
                        Sprite {
                            image: faction_icon.clone(),
                            ..default()
                        },
                        Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                            .with_translation(area_position),
                    ))
                    .id();
                population.add_token_to_area(player_entity, token);
                
                // Update player areas
                if let Ok(mut player_areas) = player_areas_query.get_mut(player_entity) {
                    player_areas.add_token_to_area(area_entity, token);
                }
            }
            
            info!("  Area {}: {} tokens for {:?}", saved_area.area_id, token_count, faction);
        }
        
        // Mark area for token position fixing
        commands.entity(area_entity).insert(FixTokenPositions);
        
        // Build city if needed
        if let Some(city_faction) = &saved_area.city_owner {
            let Some(&player_entity) = factions.0.get(city_faction) else {
                warn!("City owner faction {:?} not found", city_faction);
                continue;
            };
            
            // Spawn a fresh city token for the restored city rather than
            // popping one off `CityTokenStock`. `city_tokens_in_stock` in the
            // save already excludes tokens sitting on the board as cities, so
            // drawing from the stock here charged the player twice -- every
            // save/load round trip silently shrank their city stock by the
            // number of cities they owned, until an empty stock made
            // `recalculate_city_construction_moves_for_player` produce no
            // moves, strip `IsBuilding`, and skip them in CityConstruction.
            let city_token = commands
                .spawn(CityToken::new(player_entity))
                .id();

            commands
                .entity(area_entity)
                .insert(BuiltCity::new(city_token, player_entity));

            if let Some(city_icon) = game_factions.faction_city_icons.get(city_faction) {
                commands.entity(city_token).insert((
                    Sprite {
                        image: city_icon.clone(),
                        ..default()
                    },
                    Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                        .with_translation(area_position),
                ));
            } else {
                warn!("No city icon for faction {:?}", city_faction);
            }

            // Update player cities
            if let Ok((_, mut player_cities, _)) = player_cities_query.get_mut(player_entity) {
                player_cities.build_city_in_area(area_entity, city_token);
            }

            info!("  Area {}: city built by {:?}", saved_area.area_id, city_faction);
        }
    }
    
    // A player's city tokens are either in stock or on the board, so the stock
    // is fully derivable once the cities are placed. Recomputing it here repairs
    // saves written by the version that double-charged the stock on load -- an
    // empty stock leaves a player with no city-construction moves at all, which
    // reads as the game skipping them in that phase.
    for (player_entity, player_cities, mut city_stock) in &mut player_cities_query {
        let expected_in_stock = city_stock
            .max_tokens
            .saturating_sub(player_cities.number_of_cities());
        while city_stock.city_tokens_in_stock() < expected_in_stock {
            let token = commands.spawn(CityToken::new(player_entity)).id();
            city_stock.return_token_to_stock(token);
        }
    }

    // Clean up area-related load resources (but keep LoadingFromSave for OnEnter systems)
    commands.remove_resource::<PendingAreaPopulations>();
    commands.remove_resource::<LoadedFactionMap>();

    info!("Area populations restored from save");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::enums::GameFaction;
    use crate::civilization::{PlayerShips, ShipStock};
    use crate::stupid_ai::{IsHuman, StupidAi};
    use crate::GameActivity;
    use bevy::ecs::system::RunSystemOnce;

    /// A loaded player must come out with the same components a freshly
    /// created one gets (`setup_players` inserts `ShipStock`/`PlayerShips`
    /// via `create_ship_stock`) -- otherwise every query that requires them
    /// non-optionally (movement-move generation, the Player Info HUD's
    /// "current human" lookup) silently breaks for the rest of the loaded
    /// game.
    #[test]
    fn loaded_player_has_ship_stock_and_player_ships() {
        let mut world = World::new();
        world.init_resource::<GameInfoAndStuff>();

        let saved_player = SavedPlayer {
            name: "Test Player".to_string(),
            faction: GameFaction::Egypt,
            is_human: true,
            census_population: 0,
            treasury: 0,
            tokens_in_stock: 47,
            city_tokens_in_stock: 9,
            trade_cards: vec![],
            done_with_current_activity: false,
            ast_space: 0,
            owned_civ_cards: vec![],
        };
        world.insert_resource(PendingGameLoad(GameSaveData {
            version: SAVE_GAME_VERSION.to_string(),
            round: 1,
            game_activity: GameActivity::Movement,
            players: vec![saved_player],
            area_populations: vec![],
            census_order: vec![],
            left_to_move: vec![],
            current_mover: None,
        }));

        world.run_system_once(load_game_from_save).unwrap();

        let mut query = world.query_filtered::<Entity, (With<Player>, With<IsHuman>)>();
        let player = query.single(&world).expect("expected exactly one loaded human player");

        assert!(world.get::<ShipStock>(player).is_some(), "loaded player is missing ShipStock");
        assert!(world.get::<PlayerShips>(player).is_some(), "loaded player is missing PlayerShips");
        assert_eq!(world.get::<ShipStock>(player).unwrap().count_in_stock(), ShipStock::MAX_SHIPS);
    }

    /// Every AI move-selection system (`select_stupid_movement` et al.)
    /// requires `&Personality` non-optionally and is event-driven rather
    /// than requeryable each frame -- a loaded AI player missing this
    /// component doesn't error or skip its turn, it silently drops the
    /// `SelectStupidMove` event with no retry and never moves again,
    /// hanging that phase (and the whole game) forever. Reproduces the
    /// exact hang a save/load playthrough hit: an AI player just sits
    /// there with no available-moves processing ever happening again.
    #[test]
    fn loaded_ai_player_has_a_personality() {
        let mut world = World::new();
        world.init_resource::<GameInfoAndStuff>();

        let saved_player = SavedPlayer {
            name: "Test AI".to_string(),
            faction: GameFaction::Africa,
            is_human: false,
            census_population: 0,
            treasury: 0,
            tokens_in_stock: 47,
            city_tokens_in_stock: 9,
            trade_cards: vec![],
            done_with_current_activity: false,
            ast_space: 0,
            owned_civ_cards: vec![],
        };
        world.insert_resource(PendingGameLoad(GameSaveData {
            version: SAVE_GAME_VERSION.to_string(),
            round: 1,
            game_activity: GameActivity::Movement,
            players: vec![saved_player],
            area_populations: vec![],
            census_order: vec![],
            left_to_move: vec![],
            current_mover: None,
        }));

        world.run_system_once(load_game_from_save).unwrap();

        let mut query = world.query_filtered::<Entity, (With<Player>, With<StupidAi>)>();
        let player = query.single(&world).expect("expected exactly one loaded AI player");

        assert!(world.get::<Personality>(player).is_some(), "loaded AI player is missing Personality");
    }

    /// Rule 31.x: civilization cards owned at save time must survive a
    /// reload. Previously `PlayerCivilizationCards` wasn't part of the save
    /// format at all -- a loaded player silently owned nothing, so every
    /// card-gated bonus (Agriculture's city-reduction bonus, Medicine's
    /// Epidemic discount, Credits, etc.) quietly stopped applying after a
    /// reload with no error.
    #[test]
    fn loaded_player_keeps_their_civilization_cards() {
        let mut world = World::new();
        world.init_resource::<GameInfoAndStuff>();

        let saved_player = SavedPlayer {
            name: "Test Player".to_string(),
            faction: GameFaction::Egypt,
            is_human: true,
            census_population: 0,
            treasury: 0,
            tokens_in_stock: 47,
            city_tokens_in_stock: 9,
            trade_cards: vec![],
            done_with_current_activity: false,
            ast_space: 0,
            owned_civ_cards: vec![CivCardName::Agriculture, CivCardName::Medicine],
        };
        world.insert_resource(PendingGameLoad(GameSaveData {
            version: SAVE_GAME_VERSION.to_string(),
            round: 1,
            game_activity: GameActivity::Movement,
            players: vec![saved_player],
            area_populations: vec![],
            census_order: vec![],
            left_to_move: vec![],
            current_mover: None,
        }));

        world.run_system_once(load_game_from_save).unwrap();

        let mut query = world.query_filtered::<Entity, (With<Player>, With<IsHuman>)>();
        let player = query.single(&world).expect("expected exactly one loaded human player");

        let civ_cards = world
            .get::<PlayerCivilizationCards>(player)
            .expect("loaded player is missing PlayerCivilizationCards");
        assert!(civ_cards.owns(&CivCardName::Agriculture));
        assert!(civ_cards.owns(&CivCardName::Medicine));
        assert!(!civ_cards.owns(&CivCardName::Mining));
    }

    /// A player who owned nothing must still come out with an (empty)
    /// `PlayerCivilizationCards`, not a missing component -- several systems
    /// query it non-optionally.
    #[test]
    fn loaded_player_with_no_civ_cards_still_gets_an_empty_component() {
        let mut world = World::new();
        world.init_resource::<GameInfoAndStuff>();

        let saved_player = SavedPlayer {
            name: "Test Player".to_string(),
            faction: GameFaction::Egypt,
            is_human: true,
            census_population: 0,
            treasury: 0,
            tokens_in_stock: 47,
            city_tokens_in_stock: 9,
            trade_cards: vec![],
            done_with_current_activity: false,
            ast_space: 0,
            owned_civ_cards: vec![],
        };
        world.insert_resource(PendingGameLoad(GameSaveData {
            version: SAVE_GAME_VERSION.to_string(),
            round: 1,
            game_activity: GameActivity::Movement,
            players: vec![saved_player],
            area_populations: vec![],
            census_order: vec![],
            left_to_move: vec![],
            current_mover: None,
        }));

        world.run_system_once(load_game_from_save).unwrap();

        let mut query = world.query_filtered::<Entity, (With<Player>, With<IsHuman>)>();
        let player = query.single(&world).expect("expected exactly one loaded human player");

        let civ_cards = world
            .get::<PlayerCivilizationCards>(player)
            .expect("loaded player is missing PlayerCivilizationCards");
        assert!(civ_cards.cards.is_empty());
    }
}
