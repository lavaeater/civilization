use bevy::prelude::{Entity, Reflect};

/// State for resolving Iconoclasm and Heresy (rule 30.81).
///
/// Default: 4 cities reduced (30.811). Modifiers are cumulative (30.817):
/// - Law       : −1 city (30.812)
/// - Philosophy: −1 city (30.813)
/// - Theology  : −3 cities (30.814)
/// - Monotheism: +1 city (30.815)
/// - Road Building: +1 city (30.816)
///
/// Secondary: primary victim orders reduction of 2 other players' cities (30.818).
/// Secondary protections (30.819):
/// - Philosophy holder: cannot lose more than 1 city
/// - Theology holder: cannot be named as secondary victim
#[derive(Debug, Clone, Default, Reflect)]
pub struct IconoclasmHeresyState {
    pub phase: IconoclasmHeresyPhase,
    /// Number of the primary victim's cities to reduce. Derived from `base` (4) and
    /// `modifier` — recomputed after every `with_*` call so modifier order never
    /// affects the result (rule 30.817: cumulative, not sequential-with-clamping).
    pub cities_to_reduce: usize,
    /// Net signed adjustment to the base city count (4). Kept separate from
    /// `cities_to_reduce` so intermediate negative modifiers (e.g. Theology's −3)
    /// can't clamp to 0 before later positive modifiers (e.g. Monotheism's +1) apply.
    modifier: i32,
    /// Number of secondary-victim cities the primary victim must order reduced (always 2).
    pub secondary_cities: usize,
    pub selected_cities: Vec<Entity>,
    /// The other players' cities the primary victim has ordered reduced
    /// (30.818), once chosen.
    pub selected_secondary_cities: Vec<Entity>,
    pub immune_player: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum IconoclasmHeresyPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    SelectSecondaryVictims,
    ApplySecondaryLosses,
    Complete,
}

impl IconoclasmHeresyState {
    pub fn new() -> Self {
        Self {
            cities_to_reduce: 4,
            secondary_cities: 2,
            ..Default::default()
        }
    }

    /// Applies a signed delta to the net modifier and recomputes `cities_to_reduce`
    /// from the base (4) in one clamped step, so order of `with_*` calls can't
    /// affect the result (rule 30.817).
    fn apply_modifier(mut self, delta: i32) -> Self {
        self.modifier += delta;
        self.cities_to_reduce = (4 + self.modifier).max(0) as usize;
        self
    }

    /// Law: reduces primary cities by 1 (30.812).
    pub fn with_law(self) -> Self {
        self.apply_modifier(-1)
    }

    /// Philosophy: reduces primary cities by 1 (30.813).
    pub fn with_philosophy(self) -> Self {
        self.apply_modifier(-1)
    }

    /// Theology: reduces primary cities by 3 (30.814).
    pub fn with_theology(self) -> Self {
        self.apply_modifier(-3)
    }

    /// Monotheism: increases primary cities by 1 (30.815).
    pub fn with_monotheism(self) -> Self {
        self.apply_modifier(1)
    }

    /// Road Building: increases primary cities by 1 (30.816).
    pub fn with_road_building(self) -> Self {
        self.apply_modifier(1)
    }

    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
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

    /// Rule 30.811: four of the primary victim's cities are reduced; 30.818: two
    /// secondary-victim cities are always ordered reduced.
    #[test]
    fn base_reduces_4_primary_and_2_secondary_cities() {
        let state = IconoclasmHeresyState::new();
        assert_eq!(state.cities_to_reduce, 4);
        assert_eq!(state.secondary_cities, 2);
    }

    /// Rule 30.812/30.813: Law and Philosophy each reduce primary cities by 1.
    #[test]
    fn law_and_philosophy_each_reduce_by_1() {
        assert_eq!(IconoclasmHeresyState::new().with_law().cities_to_reduce, 3);
        assert_eq!(IconoclasmHeresyState::new().with_philosophy().cities_to_reduce, 3);
    }

    /// Rule 30.814: Theology reduces primary cities by 3.
    #[test]
    fn theology_reduces_by_3() {
        assert_eq!(IconoclasmHeresyState::new().with_theology().cities_to_reduce, 1);
    }

    /// Rule 30.815/30.816: Monotheism and Road Building each increase primary cities by 1.
    #[test]
    fn monotheism_and_road_building_each_increase_by_1() {
        assert_eq!(IconoclasmHeresyState::new().with_monotheism().cities_to_reduce, 5);
        assert_eq!(IconoclasmHeresyState::new().with_road_building().cities_to_reduce, 5);
    }

    /// Rule 30.817: all modifiers are cumulative.
    #[test]
    fn all_modifiers_cumulative() {
        // base 4: -1 (law) -1 (philosophy) -3 (theology) +1 (monotheism) +1 (road building) = 1
        let state = IconoclasmHeresyState::new()
            .with_law()
            .with_philosophy()
            .with_theology()
            .with_monotheism()
            .with_road_building();
        assert_eq!(state.cities_to_reduce, 1);
    }

    /// The reduction count cannot go negative even when modifiers overshoot.
    #[test]
    fn cannot_go_negative() {
        let state = IconoclasmHeresyState::new().with_theology().with_theology();
        assert_eq!(state.cities_to_reduce, 0); // 4 - 3 - 3, clamped at 0
    }

    /// Regression test: rule 30.817 says modifiers are cumulative, i.e. the RESULT
    /// must not depend on the order `with_*` is called in. This previously failed
    /// because each `with_*` mutated `cities_to_reduce: usize` directly with
    /// `saturating_sub`, so a negative modifier applied before a positive one could
    /// clamp to 0 and silently lose magnitude (e.g. Theology's -3 before
    /// Monotheism/RoadBuilding's +1+1 produced 2 instead of the correct 1). Fixed by
    /// accumulating a signed `modifier` and recomputing `cities_to_reduce` from the
    /// base in one clamped step.
    #[test]
    fn modifier_order_does_not_affect_result() {
        let negatives_first = IconoclasmHeresyState::new()
            .with_law()
            .with_philosophy()
            .with_theology()
            .with_monotheism()
            .with_road_building();
        let positives_first = IconoclasmHeresyState::new()
            .with_monotheism()
            .with_road_building()
            .with_theology()
            .with_law()
            .with_philosophy();
        assert_eq!(negatives_first.cities_to_reduce, 1);
        assert_eq!(positives_first.cities_to_reduce, 1);
        assert_eq!(negatives_first.cities_to_reduce, positives_first.cities_to_reduce);
    }
}
