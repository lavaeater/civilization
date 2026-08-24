use crate::civilization::enums::GameFaction;
use bevy::prelude::{Component, Entity, Resource};

/// All areas spawned by the editor carry this component as their single
/// source of truth while editing -- it mirrors `Area` from the map RON asset
/// but lives on the entity so buttons can mutate it directly. `Save` gathers
/// these (plus each entity's `Transform`) back into an `Area` list.
#[derive(Component, Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct EditableArea {
    pub id: i32,
    pub max_population: usize,
    pub land_connections: Vec<i32>,
    pub sea_connections: Vec<i32>,
    pub start_area: Option<GameFaction>,
    pub city_site: bool,
    pub flood_plain: bool,
    pub city_flood: bool,
    pub volcano: bool,
}

/// Marks every entity the editor spawns (background sprite, UI panel) that
/// isn't an `EditableArea`, so `OnExit` can despawn it uniformly.
#[derive(Component, Default)]
pub struct MapEditorRoot;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionKind {
    Land,
    Sea,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AreaFlag {
    Volcano,
    FloodPlain,
    CityFlood,
    CitySite,
}

#[derive(Resource, Default)]
pub struct MapEditorState {
    pub current_area: Option<Entity>,
    /// Set while waiting for the player to click a target area to connect to.
    pub pending_connection: Option<ConnectionKind>,
    pub dirty: bool,
    pub status: Option<String>,
    /// Set by the Save button; consumed (and cleared) by `save_map_editor`.
    pub save_requested: bool,
}

/// The 9 playable factions, in a fixed order, for the start-area cycle button.
pub const ALL_FACTIONS: [GameFaction; 9] = [
    GameFaction::Egypt,
    GameFaction::Crete,
    GameFaction::Africa,
    GameFaction::Asia,
    GameFaction::Assyria,
    GameFaction::Babylon,
    GameFaction::Illyria,
    GameFaction::Iberia,
    GameFaction::Thrace,
];

/// Cycle `current` through `None -> Egypt -> Crete -> ... -> Thrace -> None`.
pub fn next_faction_cycle(current: Option<GameFaction>) -> Option<GameFaction> {
    match current {
        None => Some(ALL_FACTIONS[0]),
        Some(f) => {
            let idx = ALL_FACTIONS.iter().position(|&x| x == f).unwrap_or(0);
            if idx + 1 >= ALL_FACTIONS.len() {
                None
            } else {
                Some(ALL_FACTIONS[idx + 1])
            }
        }
    }
}
