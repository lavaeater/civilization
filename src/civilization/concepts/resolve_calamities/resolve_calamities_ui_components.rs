use bevy::prelude::*;

/// Marks the player entity that is currently waiting for a human to complete
/// a calamity selection. The advance systems check `Has<AwaitingHumanCalamitySelection>`
/// and skip until the UI removes this component.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct AwaitingHumanCalamitySelection;

/// Shared resource used to communicate between the calamity advance systems
/// (which populate it when a human choice is needed) and the calamity UI
/// systems (which present options and write back the human's selection).
///
/// Lifecycle:
/// 1. Advance system sees `is_human` and `!awaiting_human` and `player.is_none()` →
///    calls `populate`, inserts `AwaitingHumanCalamitySelection` on the player.
/// 2. UI system spawns panel showing available cities.
/// 3. Human clicks cities and confirms → UI calls `confirm`, removes
///    `AwaitingHumanCalamitySelection` from the player.
/// 4. Next frame: advance system sees `!awaiting_human` and `player == Some(entity)` →
///    calls `take_selected_cities()`, transitions to ApplyEffects, clears resource.
#[derive(Resource, Default, Debug)]
pub struct CalamitySelectionState {
    /// The player entity whose calamity needs human input.
    pub player: Option<Entity>,
    /// All cities the player can choose from (area entities).
    pub available_cities: Vec<Entity>,
    /// Cities the human has confirmed selected.
    pub selected_cities: Vec<Entity>,
    /// How many cities must be selected to complete the action.
    pub required_count: usize,
    /// Display name of the calamity, e.g. "Civil Disorder".
    pub calamity_name: String,
    /// Navigation cursor into `available_cities`.
    pub current_index: usize,
}

impl CalamitySelectionState {
    pub fn populate(
        &mut self,
        player: Entity,
        available_cities: Vec<Entity>,
        required_count: usize,
        calamity_name: impl Into<String>,
    ) {
        self.player = Some(player);
        self.available_cities = available_cities;
        self.selected_cities.clear();
        self.required_count = required_count;
        self.calamity_name = calamity_name.into();
        self.current_index = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Remove and return the confirmed selection, then clear the resource.
    pub fn take_selected_cities(&mut self) -> Vec<Entity> {
        let cities = std::mem::take(&mut self.selected_cities);
        self.clear();
        cities
    }

    pub fn current_city(&self) -> Option<Entity> {
        self.available_cities.get(self.current_index).copied()
    }

    pub fn next(&mut self) {
        if !self.available_cities.is_empty() {
            self.current_index = (self.current_index + 1) % self.available_cities.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.available_cities.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.available_cities.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }

    pub fn toggle_current(&mut self) {
        let Some(city) = self.current_city() else { return };
        if let Some(pos) = self.selected_cities.iter().position(|&c| c == city) {
            self.selected_cities.remove(pos);
        } else if self.selected_cities.len() < self.required_count {
            self.selected_cities.push(city);
        }
    }

    pub fn is_current_selected(&self) -> bool {
        let Some(city) = self.current_city() else { return false };
        self.selected_cities.contains(&city)
    }

    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.required_count
    }
}

// ── UI component markers ──────────────────────────────────────────────────────

#[derive(Component)]
pub struct CalamitySelectionUiRoot;

#[derive(Component)]
pub struct CalamitySelectionTitleText;

#[derive(Component)]
pub struct CalamitySelectionCityNameText;

#[derive(Component)]
pub struct CalamitySelectionProgressText;

#[derive(Component)]
pub struct CalamitySelectionToggleButton;

#[derive(Component)]
pub struct CalamitySelectionConfirmButton;

#[derive(Component, Debug, Clone)]
pub enum CalamitySelectionButtonAction {
    PrevCity,
    NextCity,
    ToggleCity,
    Confirm,
}
