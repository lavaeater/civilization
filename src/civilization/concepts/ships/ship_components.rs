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
        self.ships_by_area
            .get(&area)
            .map(std::vec::Vec::as_slice)
            .unwrap_or_default()
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
        self.ships_by_area.values().map(std::vec::Vec::len).sum()
    }

    pub fn all_areas_with_ships(&self) -> Vec<Entity> {
        self.ships_by_area.keys().copied().collect()
    }
}

impl PlayerShips {
    /// Removes `ship` from `area`. Returns whether it was there.
    pub fn take_ship(&mut self, area: Entity, ship: Entity) -> bool {
        let Some(ships) = self.ships_by_area.get_mut(&area) else {
            return false;
        };
        let Some(index) = ships.iter().position(|s| *s == ship) else {
            return false;
        };
        ships.remove(index);
        if ships.is_empty() {
            self.ships_by_area.remove(&area);
        }
        true
    }
}

/// Tokens a ship may carry at one time (rule 23.51).
pub const SHIP_CAPACITY: usize = 5;

/// Water areas a ship may move into per movement phase: four, five with
/// Cloth Making (rules 23.52, 23.53).
pub fn ship_range(has_cloth_making: bool) -> usize {
    4 + usize::from(has_cloth_making)
}

/// One ship's movement so far this phase.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ShipVoyage {
    /// Water areas entered this phase.
    pub areas_moved: usize,
    /// Tokens aboard. They stand in the area the ship is in, already marked
    /// as moved; they travel with the ship until dropped off. Nothing may be
    /// aboard at the end of a phase (rule 23.56), so everything still aboard
    /// leaves the ship when its owner ends movement.
    pub cargo: Vec<Entity>,
}

/// Every ship's voyage this movement phase, by ship entity. Cleared when
/// movement starts and ends.
#[derive(bevy::prelude::Resource, Debug, Default)]
pub struct ShipVoyages {
    pub voyages: HashMap<Entity, ShipVoyage>,
}

impl ShipVoyages {
    pub fn get(&self, ship: Entity) -> ShipVoyage {
        self.voyages.get(&ship).cloned().unwrap_or_default()
    }

    pub fn areas_left(&self, ship: Entity, range: usize) -> usize {
        range.saturating_sub(self.get(ship).areas_moved)
    }

    /// The ship in `ships` to sail `cost` areas: one already carrying tokens
    /// first (it is mid-trip), then the one with the most range left.
    pub fn pick_ship(&self, ships: &[Entity], range: usize, cost: usize) -> Option<Entity> {
        ships
            .iter()
            .copied()
            .filter(|s| self.areas_left(*s, range) >= cost)
            .max_by_key(|s| (!self.get(*s).cargo.is_empty(), self.areas_left(*s, range)))
    }

    /// The most range any of `ships` has left.
    pub fn best_range_left(&self, ships: &[Entity], range: usize) -> usize {
        ships
            .iter()
            .map(|s| self.areas_left(*s, range))
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod voyage_tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn a_loaded_ship_is_used_first_and_range_runs_out() {
        let mut voyages = ShipVoyages::default();
        let (empty, loaded) = (e(1), e(2));
        voyages.voyages.insert(
            loaded,
            ShipVoyage {
                areas_moved: 2,
                cargo: vec![e(9)],
            },
        );
        let ships = [empty, loaded];
        assert_eq!(voyages.pick_ship(&ships, 4, 1), Some(loaded));
        // Three more areas are beyond the loaded ship; the empty one goes.
        assert_eq!(voyages.pick_ship(&ships, 4, 3), Some(empty));
        assert_eq!(voyages.pick_ship(&ships, 4, 5), None);
        assert_eq!(voyages.best_range_left(&ships, 5), 5);
    }
}
