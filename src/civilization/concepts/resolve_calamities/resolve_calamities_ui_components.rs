use crate::civilization::concepts::resolve_calamities::calamities::civil_war::FactionChoice;
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
        let Some(city) = self.current_city() else {
            return;
        };
        if let Some(pos) = self.selected_cities.iter().position(|&c| c == city) {
            self.selected_cities.remove(pos);
        } else if self.selected_cities.len() < self.required_count {
            self.selected_cities.push(city);
        }
    }

    pub fn is_current_selected(&self) -> bool {
        let Some(city) = self.current_city() else {
            return false;
        };
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
    /// Rule 30.415: the victim picks which of the two finalized factions to keep.
    ChooseFaction,
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
    /// Rule 30.415, role == ChooseFaction: total points in the first faction.
    pub first_faction_points: usize,
    /// Rule 30.415, role == ChooseFaction: total points in the second faction.
    pub second_faction_points: usize,
    /// Rule 30.415, role == ChooseFaction: the victim's confirmed pick.
    pub chosen_faction: Option<FactionChoice>,
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

    /// Rule 30.415: set up the victim's binary keep-first/keep-second choice.
    pub fn populate_faction_choice(
        &mut self,
        player: Entity,
        first_faction_points: usize,
        second_faction_points: usize,
    ) {
        self.acting_player = Some(player);
        self.role = CivilWarUiRole::ChooseFaction;
        self.first_faction_points = first_faction_points;
        self.second_faction_points = second_faction_points;
        self.chosen_faction = None;
    }

    pub fn choose_faction(&mut self, choice: FactionChoice) {
        self.chosen_faction = Some(choice);
    }

    /// Returns the confirmed faction choice and clears the resource.
    pub fn take_faction_choice(&mut self) -> Option<FactionChoice> {
        let choice = self.chosen_faction;
        self.clear();
        choice
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
            // Beneficiary can take anything up to target (taking 0 is also valid);
            // ChooseFaction is driven by button presses, not this token/city budget UI.
            CivilWarUiRole::Beneficiary | CivilWarUiRole::ChooseFaction => true,
        }
    }

    pub fn increment_tokens(&mut self) {
        if self.selected_token_count < self.total_available_tokens {
            let headroom = match self.role {
                CivilWarUiRole::Victim | CivilWarUiRole::ChooseFaction => usize::MAX,
                CivilWarUiRole::Beneficiary => {
                    self.target_points
                        .saturating_sub(self.selected_cities.len() * 5 + self.selected_token_count)
                        + self.selected_token_count
                }
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
        let Some(city) = self.current_city() else {
            return;
        };
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
        let Some(city) = self.current_city() else {
            return false;
        };
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

// ── Monotheism selection state ───────────────────────────────────────────────

/// Inserted on the Monotheism holder (human player) while they are choosing which
/// enemy tokens to eliminate. `apply_monotheism_conversions` waits until this is gone.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AwaitingMonotheismSelection;

/// Resource driving the human Monotheism target-selection UI.
///
/// Each candidate is a `(token_entity, area_entity)` pair; `area_entity` is used
/// for display (area name). The human picks up to 2 token entities to eliminate.
#[derive(Resource, Default, Debug)]
pub struct MonotheismSelectionState {
    /// The Monotheism holder (human player entity).
    pub player: Option<Entity>,
    /// Candidate targets: (token_entity, area_entity).
    pub candidates: Vec<(Entity, Entity)>,
    /// Token entities the human has chosen to eliminate (max 2).
    pub selected: Vec<Entity>,
    /// Navigation cursor into `candidates`.
    pub current_index: usize,
}

impl MonotheismSelectionState {
    pub fn populate(&mut self, player: Entity, candidates: Vec<(Entity, Entity)>) {
        self.player = Some(player);
        self.candidates = candidates;
        self.selected.clear();
        self.current_index = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn current_candidate(&self) -> Option<(Entity, Entity)> {
        self.candidates.get(self.current_index).copied()
    }

    pub fn next(&mut self) {
        if !self.candidates.is_empty() {
            self.current_index = (self.current_index + 1) % self.candidates.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.candidates.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.candidates.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }

    pub fn toggle_current(&mut self) {
        let Some((token, _)) = self.current_candidate() else {
            return;
        };
        if let Some(pos) = self.selected.iter().position(|&t| t == token) {
            self.selected.remove(pos);
        } else if self.selected.len() < 2 {
            self.selected.push(token);
        }
    }

    pub fn is_current_selected(&self) -> bool {
        let Some((token, _)) = self.current_candidate() else {
            return false;
        };
        self.selected.contains(&token)
    }

    /// Drain and return the selected tokens paired with the area each was
    /// found in, clearing state.
    pub fn take_result(&mut self) -> Vec<(Entity, Entity)> {
        let selected = std::mem::take(&mut self.selected);
        let results = selected
            .into_iter()
            .filter_map(|token| {
                self.candidates
                    .iter()
                    .find(|&&(t, _)| t == token)
                    .map(|&(t, area)| (t, area))
            })
            .collect();
        self.clear();
        results
    }
}

// ── Monotheism UI markers ─────────────────────────────────────────────────────

#[derive(Component, Default)]
pub struct MonotheismSelectionUiRoot;

#[derive(Component, Default)]
pub struct MonotheismTargetText;

#[derive(Component, Default)]
pub struct MonotheismProgressText;

#[derive(Component, Default)]
pub struct MonotheismToggleButton;

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

/// Rule 30.415: label on the "keep first faction" button, showing its points.
#[derive(Component)]
pub struct CivilWarKeepFirstButton;

/// Rule 30.415: label on the "keep second faction" button, showing its points.
#[derive(Component)]
pub struct CivilWarKeepSecondButton;

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
    /// Rule 30.415: victim keeps the first faction (second faction transfers).
    KeepFirstFaction,
    /// Rule 30.415: victim keeps the second faction (first faction transfers).
    KeepSecondFaction,
}

// ── Flood secondary-victim allocation state ────────────────────────────────

/// Resource that drives the Flood secondary-victim allocation UI (rule
/// 30.512): the primary victim divides a fixed point budget among the
/// secondary victims present on the flood plain, each capped at their own
/// available tokens. Only reachable when combined secondary availability
/// exceeds the budget -- if it doesn't, `allocate_secondary_loss` already
/// takes everyone's everything automatically and there's no decision to make,
/// so the advance system never populates this in that case.
///
/// The advance system populates this and inserts `AwaitingHumanCalamitySelection`
/// on the acting (primary victim) player. When the human confirms, the UI
/// removes the marker; the advance system then reads back `take_result()` and
/// passes it to `allocate_secondary_loss` as the `primary_choice`.
#[derive(Resource, Default, Debug)]
pub struct FloodSelectionState {
    pub acting_player: Option<Entity>,
    pub total_budget: usize,
    /// (secondary_victim, available_tokens, allocated_points).
    pub victims: Vec<(Entity, usize, usize)>,
    /// Navigation cursor into `victims`.
    pub current_victim_index: usize,
}

impl FloodSelectionState {
    pub fn populate(&mut self, player: Entity, victims: Vec<(Entity, usize)>, total_budget: usize) {
        self.acting_player = Some(player);
        self.total_budget = total_budget;
        self.victims = victims
            .into_iter()
            .map(|(e, available)| (e, available, 0))
            .collect();
        self.current_victim_index = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn allocated_total(&self) -> usize {
        self.victims
            .iter()
            .map(|&(_, _, allocated)| allocated)
            .sum()
    }

    pub fn remaining_budget(&self) -> usize {
        self.total_budget.saturating_sub(self.allocated_total())
    }

    pub fn current_victim(&self) -> Option<(Entity, usize, usize)> {
        self.victims.get(self.current_victim_index).copied()
    }

    pub fn next_victim(&mut self) {
        if !self.victims.is_empty() {
            self.current_victim_index = (self.current_victim_index + 1) % self.victims.len();
        }
    }

    pub fn prev_victim(&mut self) {
        if self.victims.is_empty() {
            return;
        }
        if self.current_victim_index == 0 {
            self.current_victim_index = self.victims.len() - 1;
        } else {
            self.current_victim_index -= 1;
        }
    }

    /// Adds 1 to the current victim's allocation, capped by their own
    /// availability and the remaining budget. Returns whether it changed.
    pub fn increment_current(&mut self) -> bool {
        if self.remaining_budget() == 0 {
            return false;
        }
        let idx = self.current_victim_index;
        if let Some(&mut (_, available, ref mut allocated)) = self.victims.get_mut(idx)
            && *allocated < available
        {
            *allocated += 1;
            return true;
        }
        false
    }

    /// Subtracts 1 from the current victim's allocation. Returns whether it changed.
    pub fn decrement_current(&mut self) -> bool {
        let idx = self.current_victim_index;
        if let Some(&mut (_, _, ref mut allocated)) = self.victims.get_mut(idx)
            && *allocated > 0
        {
            *allocated -= 1;
            return true;
        }
        false
    }

    /// Rule 30.512: the whole budget must be assigned somewhere among the
    /// secondary victims before confirming -- this UI is only ever shown when
    /// combined availability exceeds the budget, so full allocation is always
    /// achievable.
    pub fn selection_valid(&self) -> bool {
        self.remaining_budget() == 0
    }

    /// Returns the (victim, allocated_points) list and clears the state.
    pub fn take_result(&mut self) -> Vec<(Entity, usize)> {
        let result = self
            .victims
            .iter()
            .map(|&(e, _, allocated)| (e, allocated))
            .collect();
        self.clear();
        result
    }
}

// ── Flood UI markers ────────────────────────────────────────────────────────

#[derive(Component)]
pub struct FloodSelectionUiRoot;

#[derive(Component)]
pub struct FloodPointsText;

#[derive(Component)]
pub struct FloodVictimNameText;

#[derive(Component)]
pub struct FloodConfirmButton;

#[derive(Component, Debug, Clone)]
pub enum FloodButtonAction {
    PrevVictim,
    NextVictim,
    Increment,
    Decrement,
    Confirm,
}

// ── Famine secondary-victim allocation state ────────────────────────────────

/// Resource that drives the Famine secondary-victim allocation UI (rule
/// 30.311): the primary victim divides a fixed point budget among the
/// secondary victims sharing an area with them, each capped at their own
/// available tokens (already folded down to the 8-point-per-player cap by
/// the advance system before populating this). Only reachable when combined
/// secondary availability exceeds the budget -- if it doesn't,
/// `allocate_secondary_loss` already takes everyone's everything
/// automatically and there's no decision to make, so the advance system
/// never populates this in that case.
///
/// The advance system populates this and inserts `AwaitingHumanCalamitySelection`
/// on the acting (primary victim) player. When the human confirms, the UI
/// removes the marker; the advance system then reads back `take_result()` and
/// passes it to `allocate_secondary_loss` as the `primary_choice`.
#[derive(Resource, Default, Debug)]
pub struct FamineSelectionState {
    pub acting_player: Option<Entity>,
    pub total_budget: usize,
    /// (secondary_victim, available_tokens, allocated_points).
    pub victims: Vec<(Entity, usize, usize)>,
    /// Navigation cursor into `victims`.
    pub current_victim_index: usize,
}

impl FamineSelectionState {
    pub fn populate(&mut self, player: Entity, victims: Vec<(Entity, usize)>, total_budget: usize) {
        self.acting_player = Some(player);
        self.total_budget = total_budget;
        self.victims = victims
            .into_iter()
            .map(|(e, available)| (e, available, 0))
            .collect();
        self.current_victim_index = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn allocated_total(&self) -> usize {
        self.victims
            .iter()
            .map(|&(_, _, allocated)| allocated)
            .sum()
    }

    pub fn remaining_budget(&self) -> usize {
        self.total_budget.saturating_sub(self.allocated_total())
    }

    pub fn current_victim(&self) -> Option<(Entity, usize, usize)> {
        self.victims.get(self.current_victim_index).copied()
    }

    pub fn next_victim(&mut self) {
        if !self.victims.is_empty() {
            self.current_victim_index = (self.current_victim_index + 1) % self.victims.len();
        }
    }

    pub fn prev_victim(&mut self) {
        if self.victims.is_empty() {
            return;
        }
        if self.current_victim_index == 0 {
            self.current_victim_index = self.victims.len() - 1;
        } else {
            self.current_victim_index -= 1;
        }
    }

    /// Adds 1 to the current victim's allocation, capped by their own
    /// availability and the remaining budget. Returns whether it changed.
    pub fn increment_current(&mut self) -> bool {
        if self.remaining_budget() == 0 {
            return false;
        }
        let idx = self.current_victim_index;
        if let Some(&mut (_, available, ref mut allocated)) = self.victims.get_mut(idx)
            && *allocated < available
        {
            *allocated += 1;
            return true;
        }
        false
    }

    /// Subtracts 1 from the current victim's allocation. Returns whether it changed.
    pub fn decrement_current(&mut self) -> bool {
        let idx = self.current_victim_index;
        if let Some(&mut (_, _, ref mut allocated)) = self.victims.get_mut(idx)
            && *allocated > 0
        {
            *allocated -= 1;
            return true;
        }
        false
    }

    /// Rule 30.311: the whole budget must be assigned somewhere among the
    /// secondary victims before confirming -- this UI is only ever shown when
    /// combined availability exceeds the budget, so full allocation is always
    /// achievable.
    pub fn selection_valid(&self) -> bool {
        self.remaining_budget() == 0
    }

    /// Returns the (victim, allocated_points) list and clears the state.
    pub fn take_result(&mut self) -> Vec<(Entity, usize)> {
        let result = self
            .victims
            .iter()
            .map(|&(e, _, allocated)| (e, allocated))
            .collect();
        self.clear();
        result
    }
}

// ── Famine UI markers ───────────────────────────────────────────────────────

#[derive(Component)]
pub struct FamineSelectionUiRoot;

#[derive(Component)]
pub struct FaminePointsText;

#[derive(Component)]
pub struct FamineVictimNameText;

#[derive(Component)]
pub struct FamineConfirmButton;

#[derive(Component, Debug, Clone)]
pub enum FamineButtonAction {
    PrevVictim,
    NextVictim,
    Increment,
    Decrement,
    Confirm,
}

#[cfg(test)]
mod famine_selection_state_tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn populate_sets_up_zeroed_allocations() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 8), (e(3), 5)], 20);
        assert_eq!(state.acting_player, Some(e(1)));
        assert_eq!(state.total_budget, 20);
        assert_eq!(state.victims, vec![(e(2), 8, 0), (e(3), 5, 0)]);
        assert_eq!(state.remaining_budget(), 20);
    }

    #[test]
    fn increment_is_capped_by_remaining_budget() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 8)], 3);
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert_eq!(state.victims[0].2, 3);
        // Budget exhausted -- further increments are no-ops.
        assert!(!state.increment_current());
        assert_eq!(state.victims[0].2, 3);
    }

    #[test]
    fn increment_is_capped_by_victim_availability() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 2)], 20);
        assert!(state.increment_current());
        assert!(state.increment_current());
        // Victim only has 2 available (already capped at 8/player) -- can't
        // go higher even with budget left.
        assert!(!state.increment_current());
        assert_eq!(state.victims[0].2, 2);
        assert_eq!(state.remaining_budget(), 18);
    }

    #[test]
    fn decrement_cannot_go_below_zero() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 5)], 5);
        assert!(!state.decrement_current());
        assert_eq!(state.victims[0].2, 0);
    }

    #[test]
    fn navigation_wraps_around() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 5), (e(3), 5), (e(4), 5)], 10);
        assert_eq!(state.current_victim().unwrap().0, e(2));
        state.next_victim();
        assert_eq!(state.current_victim().unwrap().0, e(3));
        state.prev_victim();
        state.prev_victim();
        assert_eq!(state.current_victim().unwrap().0, e(4)); // wrapped backward
    }

    #[test]
    fn selection_invalid_until_full_budget_allocated() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 8), (e(3), 8), (e(4), 8)], 20);
        assert!(!state.selection_valid());
        // Each victim is capped at 8, so filling the 20-point budget requires
        // spreading across all three (8 + 8 + 4).
        for _ in 0..8 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..8 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..4 {
            state.increment_current();
        }
        assert!(state.selection_valid());
    }

    #[test]
    fn take_result_reports_allocation_and_clears_state() {
        let mut state = FamineSelectionState::default();
        state.populate(e(1), vec![(e(2), 8), (e(3), 8), (e(4), 8)], 20);
        for _ in 0..8 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..8 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..4 {
            state.increment_current();
        }
        let result = state.take_result();
        assert_eq!(result, vec![(e(2), 8), (e(3), 8), (e(4), 4)]);
        assert_eq!(state.acting_player, None);
        assert!(state.victims.is_empty());
    }
}

// ── Epidemic secondary-victim allocation state ──────────────────────────────

/// Resource that drives the Epidemic secondary-victim allocation UI (rule
/// 30.611): the primary victim divides a fixed point budget among the
/// secondary victims sharing an area with them, each capped at 10 points (5
/// with Medicine, rule 30.613). Only reachable when combined secondary caps
/// exceed the budget -- if they don't, `allocate_secondary_loss` already
/// takes everyone up to their own cap automatically and there's no decision
/// to make, so the advance system never populates this in that case.
///
/// The advance system populates this and inserts `AwaitingHumanCalamitySelection`
/// on the acting (primary victim) player. When the human confirms, the UI
/// removes the marker; the advance system then reads back `take_result()` and
/// passes it to `allocate_secondary_loss` as the `primary_choice`.
#[derive(Resource, Default, Debug)]
pub struct EpidemicSelectionState {
    pub acting_player: Option<Entity>,
    pub total_budget: usize,
    /// (secondary_victim, cap, allocated_points).
    pub victims: Vec<(Entity, usize, usize)>,
    /// Navigation cursor into `victims`.
    pub current_victim_index: usize,
}

impl EpidemicSelectionState {
    pub fn populate(&mut self, player: Entity, victims: Vec<(Entity, usize)>, total_budget: usize) {
        self.acting_player = Some(player);
        self.total_budget = total_budget;
        self.victims = victims
            .into_iter()
            .map(|(e, available)| (e, available, 0))
            .collect();
        self.current_victim_index = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn allocated_total(&self) -> usize {
        self.victims
            .iter()
            .map(|&(_, _, allocated)| allocated)
            .sum()
    }

    pub fn remaining_budget(&self) -> usize {
        self.total_budget.saturating_sub(self.allocated_total())
    }

    pub fn current_victim(&self) -> Option<(Entity, usize, usize)> {
        self.victims.get(self.current_victim_index).copied()
    }

    pub fn next_victim(&mut self) {
        if !self.victims.is_empty() {
            self.current_victim_index = (self.current_victim_index + 1) % self.victims.len();
        }
    }

    pub fn prev_victim(&mut self) {
        if self.victims.is_empty() {
            return;
        }
        if self.current_victim_index == 0 {
            self.current_victim_index = self.victims.len() - 1;
        } else {
            self.current_victim_index -= 1;
        }
    }

    /// Adds 1 to the current victim's allocation, capped by their own
    /// cap and the remaining budget. Returns whether it changed.
    pub fn increment_current(&mut self) -> bool {
        if self.remaining_budget() == 0 {
            return false;
        }
        let idx = self.current_victim_index;
        if let Some(&mut (_, available, ref mut allocated)) = self.victims.get_mut(idx)
            && *allocated < available
        {
            *allocated += 1;
            return true;
        }
        false
    }

    /// Subtracts 1 from the current victim's allocation. Returns whether it changed.
    pub fn decrement_current(&mut self) -> bool {
        let idx = self.current_victim_index;
        if let Some(&mut (_, _, ref mut allocated)) = self.victims.get_mut(idx)
            && *allocated > 0
        {
            *allocated -= 1;
            return true;
        }
        false
    }

    /// Rule 30.611: the whole budget must be assigned somewhere among the
    /// secondary victims before confirming -- this UI is only ever shown when
    /// combined caps exceed the budget, so full allocation is always
    /// achievable.
    pub fn selection_valid(&self) -> bool {
        self.remaining_budget() == 0
    }

    /// Returns the (victim, allocated_points) list and clears the state.
    pub fn take_result(&mut self) -> Vec<(Entity, usize)> {
        let result = self
            .victims
            .iter()
            .map(|&(e, _, allocated)| (e, allocated))
            .collect();
        self.clear();
        result
    }
}

// ── Epidemic UI markers ──────────────────────────────────────────────────────

#[derive(Component)]
pub struct EpidemicSelectionUiRoot;

#[derive(Component)]
pub struct EpidemicPointsText;

#[derive(Component)]
pub struct EpidemicVictimNameText;

#[derive(Component)]
pub struct EpidemicConfirmButton;

#[derive(Component, Debug, Clone)]
pub enum EpidemicButtonAction {
    PrevVictim,
    NextVictim,
    Increment,
    Decrement,
    Confirm,
}

#[cfg(test)]
mod epidemic_selection_state_tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn populate_sets_up_zeroed_allocations() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 10), (e(3), 5)], 25);
        assert_eq!(state.acting_player, Some(e(1)));
        assert_eq!(state.total_budget, 25);
        assert_eq!(state.victims, vec![(e(2), 10, 0), (e(3), 5, 0)]);
        assert_eq!(state.remaining_budget(), 25);
    }

    #[test]
    fn increment_is_capped_by_remaining_budget() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 20)], 3);
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert_eq!(state.victims[0].2, 3);
        // Budget exhausted -- further increments are no-ops.
        assert!(!state.increment_current());
        assert_eq!(state.victims[0].2, 3);
    }

    /// Rule 30.613: a secondary victim holding Medicine has a per-player cap
    /// of 5 instead of 10.
    #[test]
    fn increment_is_capped_by_victim_medicine_reduced_cap() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 5)], 25);
        for _ in 0..5 {
            assert!(state.increment_current());
        }
        // Victim's cap (Medicine-reduced to 5) is reached -- can't go higher
        // even with budget left.
        assert!(!state.increment_current());
        assert_eq!(state.victims[0].2, 5);
        assert_eq!(state.remaining_budget(), 20);
    }

    #[test]
    fn decrement_cannot_go_below_zero() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 10)], 25);
        assert!(!state.decrement_current());
        assert_eq!(state.victims[0].2, 0);
    }

    #[test]
    fn navigation_wraps_around() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 10), (e(3), 10), (e(4), 10)], 25);
        assert_eq!(state.current_victim().unwrap().0, e(2));
        state.next_victim();
        assert_eq!(state.current_victim().unwrap().0, e(3));
        state.prev_victim();
        state.prev_victim();
        assert_eq!(state.current_victim().unwrap().0, e(4)); // wrapped backward
    }

    #[test]
    fn selection_invalid_until_full_budget_allocated() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 10), (e(3), 10), (e(4), 10)], 25);
        assert!(!state.selection_valid());
        // Each victim is capped at 10, so filling the 25-point budget requires
        // spreading across all three (10 + 10 + 5).
        for _ in 0..10 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..10 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..5 {
            state.increment_current();
        }
        assert!(state.selection_valid());
    }

    #[test]
    fn take_result_reports_allocation_and_clears_state() {
        let mut state = EpidemicSelectionState::default();
        state.populate(e(1), vec![(e(2), 10), (e(3), 10), (e(4), 10)], 25);
        for _ in 0..10 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..10 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..5 {
            state.increment_current();
        }
        let result = state.take_result();
        assert_eq!(result, vec![(e(2), 10), (e(3), 10), (e(4), 5)]);
        assert_eq!(state.acting_player, None);
    }
}

#[cfg(test)]
mod flood_selection_state_tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn populate_sets_up_zeroed_allocations() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 20), (e(3), 5)], 10);
        assert_eq!(state.acting_player, Some(e(1)));
        assert_eq!(state.total_budget, 10);
        assert_eq!(state.victims, vec![(e(2), 20, 0), (e(3), 5, 0)]);
        assert_eq!(state.remaining_budget(), 10);
    }

    #[test]
    fn increment_is_capped_by_remaining_budget() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 20)], 3);
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert_eq!(state.victims[0].2, 3);
        // Budget exhausted -- further increments are no-ops.
        assert!(!state.increment_current());
        assert_eq!(state.victims[0].2, 3);
    }

    #[test]
    fn increment_is_capped_by_victim_availability() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 2)], 10);
        assert!(state.increment_current());
        assert!(state.increment_current());
        // Victim only has 2 available -- can't go higher even with budget left.
        assert!(!state.increment_current());
        assert_eq!(state.victims[0].2, 2);
        assert_eq!(state.remaining_budget(), 8);
    }

    #[test]
    fn decrement_cannot_go_below_zero() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 5)], 5);
        assert!(!state.decrement_current());
        assert_eq!(state.victims[0].2, 0);
    }

    #[test]
    fn navigation_wraps_around() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 5), (e(3), 5), (e(4), 5)], 10);
        assert_eq!(state.current_victim().unwrap().0, e(2));
        state.next_victim();
        assert_eq!(state.current_victim().unwrap().0, e(3));
        state.prev_victim();
        state.prev_victim();
        assert_eq!(state.current_victim().unwrap().0, e(4)); // wrapped backward
    }

    #[test]
    fn selection_invalid_until_full_budget_allocated() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 20), (e(3), 20)], 10);
        assert!(!state.selection_valid());
        for _ in 0..10 {
            state.increment_current();
        }
        assert!(state.selection_valid());
    }

    #[test]
    fn take_result_reports_allocation_and_clears_state() {
        let mut state = FloodSelectionState::default();
        state.populate(e(1), vec![(e(2), 20), (e(3), 20)], 10);
        for _ in 0..4 {
            state.increment_current();
        }
        state.next_victim();
        for _ in 0..6 {
            state.increment_current();
        }
        let result = state.take_result();
        assert_eq!(result, vec![(e(2), 4), (e(3), 6)]);
        assert_eq!(state.acting_player, None);
        assert!(state.victims.is_empty());
    }
}

// ── Primary unit-point loss selection (rules 29.62/29.63) ────────────────────

/// Resource driving the interactive "which of my units do I lose?" panel.
///
/// Every unit-point-loss calamity (Famine 30.311, Epidemic 30.611, ...) tells
/// the victim how *many* points to lose, never *which* units -- that choice
/// belongs to the owner of the units, and rule 29.63 requires the total to be
/// met exactly. Before this existed the loss was taken off the board in
/// arbitrary `PlayerAreas` iteration order, so a human victim watched tokens
/// vanish from areas they would never have chosen, with no prompt at all.
///
/// Availability per area is supplied by the caller, so Epidemic's
/// "leave at least one token per area" cap (30.612) is expressed simply by
/// passing `count - 1`.
///
/// Lifecycle mirrors [`FamineSelectionState`]: the advance system populates
/// this and inserts `AwaitingHumanCalamitySelection`; the UI writes the
/// allocation and removes the marker; the advance system then applies it.
#[derive(Resource, Default, Debug)]
pub struct UnitLossSelectionState {
    pub acting_player: Option<Entity>,
    /// Display name of the calamity, e.g. "Famine".
    pub calamity_name: String,
    /// Unit points that must be given up in total.
    pub total_budget: usize,
    /// (area, available, allocated).
    pub areas: Vec<(Entity, usize, usize)>,
    /// Navigation cursor into `areas`.
    pub current_area_index: usize,
    /// Cities that may be given up instead, and whether each is selected.
    /// Rule 29.62 prices a city at up to five unit points, so a victim short
    /// on tokens can (and sometimes must) meet the loss with a city.
    pub cities: Vec<(Entity, bool)>,
    /// Navigation cursor into `cities`.
    pub current_city_index: usize,
}

/// A city is worth up to five unit points when resolving a calamity (29.62).
pub const CITY_UNIT_POINTS: usize = 5;

impl UnitLossSelectionState {
    pub fn populate(
        &mut self,
        player: Entity,
        calamity_name: impl Into<String>,
        areas: Vec<(Entity, usize)>,
        cities: Vec<Entity>,
        total_budget: usize,
    ) {
        self.acting_player = Some(player);
        self.calamity_name = calamity_name.into();
        self.total_budget = total_budget;
        self.areas = areas
            .into_iter()
            .map(|(area, available)| (area, available, 0))
            .collect();
        self.current_area_index = 0;
        self.cities = cities.into_iter().map(|city| (city, false)).collect();
        self.current_city_index = 0;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn allocated_total(&self) -> usize {
        self.tokens_allocated() + self.cities_allocated() * CITY_UNIT_POINTS
    }

    pub fn tokens_allocated(&self) -> usize {
        self.areas.iter().map(|&(_, _, allocated)| allocated).sum()
    }

    pub fn cities_allocated(&self) -> usize {
        self.cities
            .iter()
            .filter(|&&(_, selected)| selected)
            .count()
    }

    pub fn total_available(&self) -> usize {
        self.tokens_available() + self.cities.len() * CITY_UNIT_POINTS
    }

    pub fn selected_cities(&self) -> Vec<Entity> {
        self.cities
            .iter()
            .filter(|&&(_, selected)| selected)
            .map(|&(city, _)| city)
            .collect()
    }

    pub fn current_city(&self) -> Option<(Entity, bool)> {
        self.cities.get(self.current_city_index).copied()
    }

    pub fn next_city(&mut self) {
        if !self.cities.is_empty() {
            self.current_city_index = (self.current_city_index + 1) % self.cities.len();
        }
    }

    pub fn prev_city(&mut self) {
        if self.cities.is_empty() {
            return;
        }
        if self.current_city_index == 0 {
            self.current_city_index = self.cities.len() - 1;
        } else {
            self.current_city_index -= 1;
        }
    }

    /// Toggles whether the currently shown city is given up.
    pub fn toggle_current_city(&mut self) -> bool {
        let idx = self.current_city_index;
        if let Some(&mut (_, ref mut selected)) = self.cities.get_mut(idx) {
            *selected = !*selected;
            return true;
        }
        false
    }

    /// The amount that actually has to be given up: the budget, or everything
    /// the player has if that is less (rule 29.63 -- you cannot lose what you
    /// do not have).
    pub fn required_total(&self) -> usize {
        self.total_budget.min(self.total_available())
    }

    pub fn remaining(&self) -> usize {
        self.minimal_valid_total()
            .saturating_sub(self.allocated_total())
    }

    pub fn current_area(&self) -> Option<(Entity, usize, usize)> {
        self.areas.get(self.current_area_index).copied()
    }

    pub fn next_area(&mut self) {
        if !self.areas.is_empty() {
            self.current_area_index = (self.current_area_index + 1) % self.areas.len();
        }
    }

    pub fn prev_area(&mut self) {
        if self.areas.is_empty() {
            return;
        }
        if self.current_area_index == 0 {
            self.current_area_index = self.areas.len() - 1;
        } else {
            self.current_area_index -= 1;
        }
    }

    /// Adds 1 to the current area's allocation, capped by what is in that area
    /// and by the points still owed. Returns whether it changed.
    pub fn increment_current(&mut self) -> bool {
        if self.remaining() == 0 {
            return false;
        }
        let idx = self.current_area_index;
        if let Some(&mut (_, available, ref mut allocated)) = self.areas.get_mut(idx)
            && *allocated < available
        {
            *allocated += 1;
            return true;
        }
        false
    }

    /// Subtracts 1 from the current area's allocation. Returns whether it changed.
    pub fn decrement_current(&mut self) -> bool {
        let idx = self.current_area_index;
        if let Some(&mut (_, _, ref mut allocated)) = self.areas.get_mut(idx)
            && *allocated > 0
        {
            *allocated -= 1;
            return true;
        }
        false
    }

    /// The total the victim must hand over, per rule 29.63: "Players must
    /// fulfil calamity losses by the exact amount required. If unable, they
    /// may exceed the amount, but only by as small an amount as necessary."
    ///
    /// Tokens are 1 point each, so any amount within reach of the tokens
    /// alone can always be met exactly and no overshoot is permitted. Only
    /// when the tokens fall short must cities (5 points each, 29.62) make up
    /// the difference, and then the fewest possible are used -- which fixes
    /// the total even though *which* tokens and cities go is still the
    /// victim's choice.
    pub fn minimal_valid_total(&self) -> usize {
        let required = self.required_total();
        let tokens = self.tokens_available();
        if required <= tokens {
            return required;
        }
        let cities_needed = (required - tokens).div_ceil(CITY_UNIT_POINTS);
        required.max(cities_needed * CITY_UNIT_POINTS)
    }

    pub fn tokens_available(&self) -> usize {
        self.areas.iter().map(|&(_, available, _)| available).sum()
    }

    pub fn selection_valid(&self) -> bool {
        self.allocated_total() == self.minimal_valid_total()
    }

    /// Remove and return the confirmed loss -- per-area token counts and the
    /// cities given up -- then clear.
    pub fn take_allocation(&mut self) -> (Vec<(Entity, usize)>, Vec<Entity>) {
        let tokens = self
            .areas
            .iter()
            .filter(|&&(_, _, allocated)| allocated > 0)
            .map(|&(area, _, allocated)| (area, allocated))
            .collect();
        let cities = self.selected_cities();
        self.clear();
        (tokens, cities)
    }
}

#[derive(Component)]
pub struct UnitLossSelectionUiRoot;

#[derive(Component)]
pub struct UnitLossTitleText;

#[derive(Component)]
pub struct UnitLossPointsText;

#[derive(Component)]
pub struct UnitLossAreaNameText;

#[derive(Component)]
pub struct UnitLossCityText;

#[derive(Component)]
pub struct UnitLossConfirmButton;

#[derive(Component, Debug, Clone)]
pub enum UnitLossButtonAction {
    PrevArea,
    NextArea,
    Increment,
    Decrement,
    PrevCity,
    NextCity,
    ToggleCity,
    Confirm,
}

#[cfg(test)]
mod unit_loss_selection_state_tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn allocation_is_capped_by_area_availability() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 2), (e(3), 9)], Vec::new(), 10);

        // Area e(2) only holds 2 tokens, so the third increment must not take.
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert!(!state.increment_current());
        assert_eq!(state.areas[0].2, 2);
    }

    #[test]
    fn allocation_is_capped_by_the_budget() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 50)], Vec::new(), 3);

        assert!(state.increment_current());
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert!(!state.increment_current(), "budget of 3 is spent");
        assert_eq!(state.allocated_total(), 3);
        assert!(state.selection_valid());
    }

    /// Rule 29.63: the required total is capped by what the player actually
    /// has, so a victim with fewer units than the loss can still confirm.
    #[test]
    fn required_total_is_capped_by_what_the_player_owns() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Epidemic", vec![(e(2), 1), (e(3), 2)], Vec::new(), 16);

        assert_eq!(state.required_total(), 3);
        assert!(state.increment_current(), "1 token in the first area");
        assert!(!state.increment_current(), "first area is exhausted");
        state.next_area();
        assert!(state.increment_current());
        assert!(state.increment_current());
        assert_eq!(state.allocated_total(), 3);
        assert!(state.selection_valid(), "losing everything satisfies 29.63");
    }

    #[test]
    fn confirming_requires_the_full_amount() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 5), (e(3), 5)], Vec::new(), 4);

        assert!(!state.selection_valid());
        state.increment_current();
        assert!(!state.selection_valid(), "1 of 4 assigned");
        state.increment_current();
        state.increment_current();
        state.increment_current();
        assert!(state.selection_valid());
    }

    /// Rule 29.62 prices a city at five unit points, so a victim short on
    /// tokens can settle the loss with one.
    #[test]
    fn a_city_covers_five_unit_points() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 2)], vec![e(3)], 7);

        assert_eq!(state.total_available(), 7, "2 tokens + a 5-point city");
        assert_eq!(state.required_total(), 7);

        state.increment_current();
        state.increment_current();
        assert!(!state.selection_valid(), "2 of 7 points assigned");

        state.toggle_current_city();
        assert_eq!(state.allocated_total(), 7);
        assert!(state.selection_valid());
        assert_eq!(state.take_allocation(), (vec![(e(2), 2)], vec![e(3)]));
    }

    /// Rule 29.63: an overshoot is allowed when the amount cannot be met
    /// exactly, but only by as little as necessary. A 5-point city covering a
    /// 3-point shortfall is fine; a second, redundant city is not.
    #[test]
    fn an_overshoot_is_valid_only_when_it_cannot_be_trimmed() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![], vec![e(2), e(3)], 3);

        assert!(!state.selection_valid(), "nothing assigned yet");

        state.toggle_current_city();
        assert_eq!(state.allocated_total(), 5);
        assert!(
            state.selection_valid(),
            "5 for a 3-point loss: no smaller option exists"
        );

        state.next_city();
        state.toggle_current_city();
        assert_eq!(state.allocated_total(), 10);
        assert!(!state.selection_valid(), "the second city is redundant");
    }

    /// Tokens are preferred where they can meet the amount exactly -- giving
    /// a city instead would overshoot avoidably.
    #[test]
    fn a_city_may_not_be_given_when_tokens_cover_the_loss_exactly() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 9)], vec![e(3)], 3);

        state.toggle_current_city();
        assert!(
            !state.selection_valid(),
            "5 points for a 3-point loss with tokens available"
        );

        state.toggle_current_city();
        for _ in 0..3 {
            state.increment_current();
        }
        assert!(state.selection_valid());
        assert_eq!(state.take_allocation(), (vec![(e(2), 3)], Vec::new()));
    }

    #[test]
    fn take_allocation_returns_only_areas_with_losses_and_clears() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 5), (e(3), 5)], Vec::new(), 2);

        state.next_area();
        state.increment_current();
        state.increment_current();

        assert_eq!(state.take_allocation(), (vec![(e(3), 2)], Vec::new()));
        assert_eq!(state.acting_player, None);
        assert!(state.areas.is_empty());
    }

    #[test]
    fn decrement_frees_budget_for_another_area() {
        let mut state = UnitLossSelectionState::default();
        state.populate(e(1), "Famine", vec![(e(2), 5), (e(3), 5)], Vec::new(), 1);

        assert!(state.increment_current());
        state.next_area();
        assert!(!state.increment_current(), "budget exhausted");
        state.prev_area();
        assert!(state.decrement_current());
        state.next_area();
        assert!(state.increment_current());
        assert_eq!(state.take_allocation(), (vec![(e(3), 1)], Vec::new()));
    }
}
