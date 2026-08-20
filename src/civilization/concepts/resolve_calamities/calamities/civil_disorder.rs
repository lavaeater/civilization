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

#[cfg(test)]
mod tests {
    use super::*;

    /// Rule 30.711: all but 3 of the primary victim's cities are reduced.
    #[test]
    fn base_keeps_3_cities() {
        let mut state = CivilDisorderState::new();
        state.compute_cities_to_reduce(8);
        assert_eq!(state.cities_to_reduce, 5); // 8 - 3
    }

    /// Rule 30.712: Music, Drama and Poetry, Law, and Democracy each keep +1 (cumulative, 30.715).
    #[test]
    fn keep_modifiers_are_cumulative() {
        let mut state = CivilDisorderState::new()
            .with_music()
            .with_drama_and_poetry()
            .with_law()
            .with_democracy();
        state.compute_cities_to_reduce(8);
        assert_eq!(state.cities_to_reduce, 1); // keep 3+4=7, reduce 8-7=1
    }

    /// Rule 30.713/30.714: Military and Road Building each reduce the keep threshold by 1.
    #[test]
    fn military_and_road_building_reduce_keep_threshold() {
        let mut state = CivilDisorderState::new()
            .with_military()
            .with_road_building();
        state.compute_cities_to_reduce(8);
        assert_eq!(state.cities_to_reduce, 7); // keep 3-2=1, reduce 8-1=7
    }

    /// All six modifiers together, per rule 30.715 (cumulative).
    #[test]
    fn all_modifiers_cumulative() {
        let mut state = CivilDisorderState::new()
            .with_music()
            .with_drama_and_poetry()
            .with_law()
            .with_democracy()
            .with_military()
            .with_road_building();
        // +1+1+1+1-1-1 = +2 net -> keep 3+2=5
        state.compute_cities_to_reduce(8);
        assert_eq!(state.cities_to_reduce, 3); // 8 - 5
    }

    /// The keep threshold cannot go negative even with many negative modifiers.
    #[test]
    fn keep_threshold_does_not_go_negative() {
        let mut state = CivilDisorderState::new()
            .with_military()
            .with_road_building()
            .with_military() // stack a third negative modifier to push below 0
            .with_road_building();
        state.compute_cities_to_reduce(3);
        assert_eq!(state.cities_to_reduce, 3); // keep clamps at 0, so all 3 are reduced
    }

    /// Cannot reduce more cities than the player actually has.
    #[test]
    fn cannot_reduce_more_cities_than_owned() {
        let mut state = CivilDisorderState::new();
        state.compute_cities_to_reduce(2); // fewer than the 3 normally kept
        assert_eq!(state.cities_to_reduce, 0);
    }
}
