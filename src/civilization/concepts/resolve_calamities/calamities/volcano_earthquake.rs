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
