use bevy::prelude::*;

/// Inserted on a human player entity while we are waiting for them to decide
/// how many ships to build and where to place them. The `advance_ship_construction`
/// system will not transition to `Movement` until this marker is gone.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AwaitingShipPlacement;

/// Resource that drives the human ship-construction UI.
///
/// Lifecycle:
/// 1. `enter_ship_construction` sees a human player, populates this and inserts
///    `AwaitingShipPlacement` on them.
/// 2. `spawn_ship_construction_ui` detects the marker and shows the panel.
/// 3. Human adjusts count / area and clicks Confirm.
/// 4. `handle_ship_construction_buttons` writes the result, removes the marker.
/// 5. `advance_ship_construction` sees no more waiting players → transitions.
#[derive(Resource, Default, Debug)]
pub struct ShipConstructionState {
    /// The human player entity waiting for input.
    pub player: Option<Entity>,
    /// Areas where the player currently has tokens (eligible ship placements).
    pub available_areas: Vec<Entity>,
    /// How many ships the player wants to build (0..=max_buildable).
    pub ships_to_build: usize,
    /// Upper bound on ships this player can build (limited by stock and treasury/tokens).
    pub max_buildable: usize,
    /// For each ship slot the player wants to build, which area to place it in.
    /// Length is always clamped to `ships_to_build`.
    pub selected_areas: Vec<Entity>,
    /// Navigation cursor: which ship slot is currently being configured (0-indexed).
    pub current_slot: usize,
    /// Navigation cursor into `available_areas` for the current slot.
    pub area_cursor: usize,
}

impl ShipConstructionState {
    pub fn populate(
        &mut self,
        player: Entity,
        available_areas: Vec<Entity>,
        max_buildable: usize,
    ) {
        self.player = Some(player);
        self.available_areas = available_areas.clone();
        self.ships_to_build = 0;
        self.max_buildable = max_buildable;
        self.selected_areas.clear();
        self.current_slot = 0;
        self.area_cursor = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Increase ship count by 1, adding a default area selection.
    pub fn increment(&mut self) {
        if self.ships_to_build < self.max_buildable && !self.available_areas.is_empty() {
            self.ships_to_build += 1;
            self.selected_areas.push(self.available_areas[0]);
        }
    }

    /// Decrease ship count by 1.
    pub fn decrement(&mut self) {
        if self.ships_to_build > 0 {
            self.ships_to_build -= 1;
            self.selected_areas.pop();
            if self.current_slot >= self.ships_to_build && self.current_slot > 0 {
                self.current_slot -= 1;
            }
        }
    }

    /// The area currently shown for `current_slot`.
    pub fn current_area(&self) -> Option<Entity> {
        self.available_areas.get(self.area_cursor).copied()
    }

    pub fn next_area(&mut self) {
        if !self.available_areas.is_empty() {
            self.area_cursor = (self.area_cursor + 1) % self.available_areas.len();
            self.sync_area_to_slot();
        }
    }

    pub fn prev_area(&mut self) {
        if !self.available_areas.is_empty() {
            if self.area_cursor == 0 {
                self.area_cursor = self.available_areas.len() - 1;
            } else {
                self.area_cursor -= 1;
            }
            self.sync_area_to_slot();
        }
    }

    pub fn next_slot(&mut self) {
        if self.ships_to_build > 0 {
            self.current_slot = (self.current_slot + 1) % self.ships_to_build;
            self.sync_cursor_to_slot();
        }
    }

    pub fn prev_slot(&mut self) {
        if self.ships_to_build > 0 {
            if self.current_slot == 0 {
                self.current_slot = self.ships_to_build - 1;
            } else {
                self.current_slot -= 1;
            }
            self.sync_cursor_to_slot();
        }
    }

    fn sync_area_to_slot(&mut self) {
        if let Some(area) = self.available_areas.get(self.area_cursor).copied()
            && let Some(slot) = self.selected_areas.get_mut(self.current_slot)
        {
            *slot = area;
        }
    }

    fn sync_cursor_to_slot(&mut self) {
        if let Some(&area) = self.selected_areas.get(self.current_slot) {
            if let Some(idx) = self.available_areas.iter().position(|&a| a == area) {
                self.area_cursor = idx;
            } else {
                self.area_cursor = 0;
            }
        }
    }

    /// Returns the build plan (area per ship) and clears the resource.
    pub fn take_result(&mut self) -> Vec<Entity> {
        let areas = std::mem::take(&mut self.selected_areas);
        self.clear();
        areas
    }
}

// ── UI marker components ──────────────────────────────────────────────────────

#[derive(Component, Default)]
pub struct ShipConstructionUiRoot;

#[derive(Component, Default)]
pub struct ShipCountText;

#[derive(Component, Default)]
pub struct ShipSlotText;

#[derive(Component, Default)]
pub struct ShipAreaText;

