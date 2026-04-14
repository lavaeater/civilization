use bevy::prelude::{Entity, Reflect};

/// State for Barbarian Hordes (rule 30.52).
///
/// Visual barbarian token entities are spawned in the landing area during
/// `PlaceBarbarians` and despawned in `Complete`.
#[derive(Debug, Clone, Default, Reflect)]
pub struct BarbarianHordesState {
    pub phase: BarbarianHordesPhase,
    /// Unit points the primary victim loses (base 15; Military −5).
    pub unit_points_to_lose: i32,
    /// Start area where Barbarians land (the one with most victim units).
    pub landing_area: Option<Entity>,
    pub has_military: bool,
    /// Visual barbarian token entities currently on the map.
    pub barbarian_tokens: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum BarbarianHordesPhase {
    #[default]
    FindLandingArea,
    /// Spawn visual barbarian token entities in the landing area.
    PlaceBarbarians,
    ApplyEffects,
    Complete,
}

impl BarbarianHordesState {
    pub fn new() -> Self {
        Self {
            unit_points_to_lose: 15,
            ..Default::default()
        }
    }

    pub fn with_military(mut self) -> Self {
        self.has_military = true;
        self.unit_points_to_lose = (self.unit_points_to_lose - 5).max(0);
        self
    }
}
