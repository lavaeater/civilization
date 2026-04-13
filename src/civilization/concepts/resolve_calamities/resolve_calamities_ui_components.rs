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

// ── Civil War selection state ─────────────────────────────────────────────────

/// Whether the acting player is the Civil War victim or beneficiary.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum CivilWarUiRole {
    #[default]
    Victim,
    Beneficiary,
}

/// Resource that drives the Civil War interactive selection UI.
///
/// The advance system populates this and inserts `AwaitingHumanCalamitySelection`
/// on the acting player.  When the human confirms, the UI removes the marker;
/// the advance system then reads back `take_tokens()` / `take_cities()` and clears.
#[derive(Resource, Default, Debug)]
pub struct CivilWarSelectionState {
    pub acting_player: Option<Entity>,
    pub role: CivilWarUiRole,
    /// How many points the player must reach (victim: at least; beneficiary: at most).
    pub target_points: usize,
    /// Total tokens available to draw from.
    pub total_available_tokens: usize,
    /// How many tokens the acting player has chosen.
    pub selected_token_count: usize,
    /// Cities available to toggle (victim: own cities; beneficiary: victim's selected cities).
    pub available_cities: Vec<Entity>,
    /// Cities the acting player has toggled in.
    pub selected_cities: Vec<Entity>,
    /// Navigation cursor into available_cities.
    pub current_city_index: usize,
    /// Whether the UI is showing the city list (true) or the token spinner (false).
    pub showing_cities: bool,
}

impl CivilWarSelectionState {
    pub fn populate_victim(
        &mut self,
        player: Entity,
        total_tokens: usize,
        cities: Vec<Entity>,
        target: usize,
    ) {
        self.acting_player = Some(player);
        self.role = CivilWarUiRole::Victim;
        self.target_points = target;
        self.total_available_tokens = total_tokens;
        self.selected_token_count = 0;
        self.available_cities = cities;
        self.selected_cities.clear();
        self.current_city_index = 0;
        self.showing_cities = false;
    }

    pub fn populate_beneficiary(
        &mut self,
        player: Entity,
        pool_tokens: usize,
        pool_cities: Vec<Entity>,
        target: usize,
    ) {
        self.acting_player = Some(player);
        self.role = CivilWarUiRole::Beneficiary;
        self.target_points = target;
        self.total_available_tokens = pool_tokens;
        self.selected_token_count = 0;
        self.available_cities = pool_cities;
        self.selected_cities.clear();
        self.current_city_index = 0;
        self.showing_cities = false;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn current_points(&self) -> usize {
        self.selected_token_count + self.selected_cities.len() * 5
    }

    pub fn selection_valid(&self) -> bool {
        match self.role {
            // Victim must meet or exceed target
            CivilWarUiRole::Victim => self.current_points() >= self.target_points,
            // Beneficiary can take anything up to target (taking 0 is also valid)
            CivilWarUiRole::Beneficiary => true,
        }
    }

    pub fn increment_tokens(&mut self) {
        if self.selected_token_count < self.total_available_tokens {
            let headroom = match self.role {
                CivilWarUiRole::Victim => usize::MAX,
                CivilWarUiRole::Beneficiary => self.target_points.saturating_sub(
                    self.selected_cities.len() * 5 + self.selected_token_count,
                ) + self.selected_token_count,
            };
            if self.selected_token_count < headroom {
                self.selected_token_count += 1;
            }
        }
    }

    pub fn decrement_tokens(&mut self) {
        if self.selected_token_count > 0 {
            self.selected_token_count -= 1;
        }
    }

    pub fn current_city(&self) -> Option<Entity> {
        self.available_cities.get(self.current_city_index).copied()
    }

    pub fn next_city(&mut self) {
        if !self.available_cities.is_empty() {
            self.current_city_index = (self.current_city_index + 1) % self.available_cities.len();
        }
    }

    pub fn prev_city(&mut self) {
        if !self.available_cities.is_empty() {
            if self.current_city_index == 0 {
                self.current_city_index = self.available_cities.len() - 1;
            } else {
                self.current_city_index -= 1;
            }
        }
    }

    pub fn toggle_current_city(&mut self) {
        let Some(city) = self.current_city() else { return };
        if let Some(pos) = self.selected_cities.iter().position(|&c| c == city) {
            self.selected_cities.remove(pos);
        } else {
            // Beneficiary: don't exceed target
            if self.role == CivilWarUiRole::Beneficiary
                && self.current_points() + 5 > self.target_points
            {
                return;
            }
            self.selected_cities.push(city);
        }
    }

    pub fn is_current_city_selected(&self) -> bool {
        let Some(city) = self.current_city() else { return false };
        self.selected_cities.contains(&city)
    }

    /// Returns selected token count and clears the state.
    pub fn take_result(&mut self) -> (usize, Vec<Entity>) {
        let tokens = self.selected_token_count;
        let cities = std::mem::take(&mut self.selected_cities);
        self.clear();
        (tokens, cities)
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

// ── Civil War UI markers ──────────────────────────────────────────────────────

#[derive(Component)]
pub struct CivilWarSelectionUiRoot;

#[derive(Component)]
pub struct CivilWarTitleText;

#[derive(Component)]
pub struct CivilWarPointsText;

#[derive(Component)]
pub struct CivilWarTokenCountText;

#[derive(Component)]
pub struct CivilWarCityNameText;

#[derive(Component)]
pub struct CivilWarToggleCityButton;

#[derive(Component)]
pub struct CivilWarConfirmButton;

#[derive(Component, Debug, Clone)]
pub enum CivilWarButtonAction {
    TokensTab,
    CitiesTab,
    IncrementTokens,
    DecrementTokens,
    PrevCity,
    NextCity,
    ToggleCity,
    Confirm,
}
