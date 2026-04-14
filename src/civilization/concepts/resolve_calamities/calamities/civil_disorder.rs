use bevy::prelude::{Entity, Reflect};

/// State for resolving Civil Disorder (rule 30.71).
///
/// Default: all but 3 of the primary victim's cities are reduced (30.711).
/// `cities_modifier` is a signed adjustment to the base "keep 3" count.
/// Positive modifier means more cities kept (fewer reduced); negative means more reduced.
///
/// Modifiers (cumulative, 30.715):
/// - Music           : keep +1 (30.712)
/// - Drama and Poetry: keep +1 (30.712)
/// - Law             : keep +1 (30.712)
/// - Democracy       : keep +1 (30.712)
/// - Military        : keep −1 (more reduced, 30.713)
/// - Road Building   : keep −1 (more reduced, 30.714)
#[derive(Debug, Clone, Default, Reflect)]
pub struct CivilDisorderState {
    pub phase: CivilDisorderPhase,
    /// Computed at ComputeEffects time via `compute_cities_to_reduce`.
    pub cities_to_reduce: usize,
    /// Net signed adjustment to the "keep" threshold (base = 3).
    pub cities_modifier: i32,
    pub selected_cities: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum CivilDisorderPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    ApplyEffects,
    Complete,
}

impl CivilDisorderState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute the number of cities to reduce given the player's current city total.
    /// Call this once at the start of the ComputeEffects phase.
    pub fn compute_cities_to_reduce(&mut self, total_cities: usize) {
        let keep = (3i32 + self.cities_modifier).max(0) as usize;
        self.cities_to_reduce = total_cities.saturating_sub(keep);
    }

    /// Music: −1 city reduced (keep +1, 30.712).
    pub fn with_music(mut self) -> Self {
        self.cities_modifier += 1;
        self
    }

    /// Drama and Poetry: −1 city reduced (keep +1, 30.712).
    pub fn with_drama_and_poetry(mut self) -> Self {
        self.cities_modifier += 1;
        self
    }

    /// Law: −1 city reduced (keep +1, 30.712).
    pub fn with_law(mut self) -> Self {
        self.cities_modifier += 1;
        self
    }

    /// Democracy: −1 city reduced (keep +1, 30.712).
    pub fn with_democracy(mut self) -> Self {
        self.cities_modifier += 1;
        self
    }

    /// Military: +1 city reduced (keep −1, 30.713).
    pub fn with_military(mut self) -> Self {
        self.cities_modifier -= 1;
        self
    }

    /// Road Building: +1 city reduced (keep −1, 30.714).
    pub fn with_road_building(mut self) -> Self {
        self.cities_modifier -= 1;
        self
    }

    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.cities_to_reduce {
            self.selected_cities.push(city_area);
        }
    }

    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.cities_to_reduce
    }
}
