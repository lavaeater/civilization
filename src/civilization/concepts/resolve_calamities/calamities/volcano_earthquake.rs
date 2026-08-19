use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct VolcanoEarthquakeState {
    pub phase: VolcanoEarthquakePhase,
    pub is_volcano: bool,
    pub volcano_area: Option<Entity>,
    pub city_to_destroy: Option<Entity>,
    pub city_to_reduce: Option<Entity>,
    pub areas_to_clear: Vec<Entity>,
    pub has_engineering: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum VolcanoEarthquakePhase {
    #[default]
    DetermineType,
    FindTargets,
    ApplyEffects,
    Complete,
}

impl VolcanoEarthquakeState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn as_volcano(volcano_area: Entity, areas_to_clear: Vec<Entity>) -> Self {
        Self {
            phase: VolcanoEarthquakePhase::ApplyEffects,
            is_volcano: true,
            volcano_area: Some(volcano_area),
            areas_to_clear,
            ..Default::default()
        }
    }
    
    pub fn as_earthquake(city_to_destroy: Entity, city_to_reduce: Option<Entity>, has_engineering: bool) -> Self {
        Self {
            phase: VolcanoEarthquakePhase::ApplyEffects,
            is_volcano: false,
            city_to_destroy: if has_engineering { None } else { Some(city_to_destroy) },
            city_to_reduce: if has_engineering { Some(city_to_destroy) } else { city_to_reduce },
            has_engineering,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rule 30.211: a volcano eruption eliminates all units in every touched area.
    #[test]
    fn volcano_carries_all_touched_areas() {
        let volcano_area = crate::test_utils::create_test_entity();
        let adjacent = crate::test_utils::create_test_entity();
        let state = VolcanoEarthquakeState::as_volcano(volcano_area, vec![volcano_area, adjacent]);
        assert!(state.is_volcano);
        assert_eq!(state.volcano_area, Some(volcano_area));
        assert_eq!(state.areas_to_clear, vec![volcano_area, adjacent]);
    }

    /// Rule 30.212: without Engineering, the earthquake DESTROYS the victim's city.
    #[test]
    fn earthquake_without_engineering_destroys_city() {
        let city = crate::test_utils::create_test_entity();
        let state = VolcanoEarthquakeState::as_earthquake(city, None, false);
        assert!(!state.is_volcano);
        assert_eq!(state.city_to_destroy, Some(city));
        assert_eq!(state.city_to_reduce, None);
    }

    /// Rule 30.213: Engineering downgrades destroy-to-reduce for the primary victim's
    /// own city — the adjacent secondary city (if any) is untouched by this swap.
    #[test]
    fn engineering_downgrades_destroy_to_reduce() {
        let city = crate::test_utils::create_test_entity();
        let secondary = crate::test_utils::create_test_entity();
        let state = VolcanoEarthquakeState::as_earthquake(city, Some(secondary), true);
        assert_eq!(state.city_to_destroy, None);
        assert_eq!(state.city_to_reduce, Some(city));
        assert!(state.has_engineering);
    }

    // NOTE: rule 30.213 also says an Engineering holder may not be *selected* as
    // the secondary victim at all (immunity, not just a destroy->reduce downgrade
    // on their own city). That selection-time exclusion is enforced in
    // `find_adjacent_city_to_reduce` (resolve_calamities_systems.rs), which needs
    // a live ECS query over other players' civ cards — not testable from this pure
    // state struct. See resolve_calamities_tests.rs / a future ECS-level test for
    // that half of the rule.
}
