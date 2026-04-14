use bevy::platform::collections::HashMap;
use bevy::prelude::{Component, Entity, Reflect, ReflectComponent};

/// Marker component on a ship entity.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Ship {
    /// The player who owns this ship.
    pub owner: Entity,
}

impl Ship {
    pub fn new(owner: Entity) -> Self {
        Self { owner }
    }
}

/// Component on a player entity tracking ships NOT currently on the board.
/// Maximum 4 ships per player at any time (rule 22.4).
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ShipStock {
    pub ships: Vec<Entity>,
}

impl ShipStock {
    pub const MAX_SHIPS: usize = 4;

    pub fn new(ships: Vec<Entity>) -> Self {
        Self { ships }
    }

    pub fn take_ship(&mut self) -> Option<Entity> {
        self.ships.pop()
    }

    pub fn return_ship(&mut self, ship: Entity) {
        self.ships.push(ship);
    }

    pub fn count_in_stock(&self) -> usize {
        self.ships.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ships.is_empty()
    }
}

/// Component on a player entity tracking ships currently placed on the board.
/// Maps coastal `area_entity → Vec<ship_entity>`.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerShips {
    pub ships_by_area: HashMap<Entity, Vec<Entity>>,
}

impl PlayerShips {
    pub fn ships_in_area(&self, area: Entity) -> &[Entity] {
        self.ships_by_area.get(&area).map(|v| v.as_slice()).unwrap_or_default()
    }

    pub fn place_ship(&mut self, area: Entity, ship: Entity) {
        self.ships_by_area.entry(area).or_default().push(ship);
    }

    /// Removes one ship from `area`, returning it. Returns `None` if no ship there.
    pub fn remove_ship_from_area(&mut self, area: Entity) -> Option<Entity> {
        if let Some(ships) = self.ships_by_area.get_mut(&area) {
            let ship = ships.pop();
            if ships.is_empty() {
                self.ships_by_area.remove(&area);
            }
            ship
        } else {
            None
        }
    }

    pub fn total_ships_on_board(&self) -> usize {
        self.ships_by_area.values().map(|v| v.len()).sum()
    }

    pub fn all_areas_with_ships(&self) -> Vec<Entity> {
        self.ships_by_area.keys().cloned().collect()
    }
}
