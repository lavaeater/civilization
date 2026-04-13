use bevy::prelude::{Entity, Reflect};

/// State for Barbarian Hordes (rule 30.52).
///
/// Implementation: instead of persistent Barbarian entities, we find the primary
/// victim's start area causing the greatest damage and remove tokens from there
/// outward (area + adjacent areas). This correctly captures geographic targeting
/// without requiring a separate Barbarian faction entity.
#[derive(Debug, Clone, Default, Reflect)]
pub struct BarbarianHordesState {
    pub phase: BarbarianHordesPhase,
    /// Unit points the primary victim loses (base 15; Military −5).
    pub unit_points_to_lose: i32,
    /// Start area where Barbarians land (the one with most victim units).
    pub landing_area: Option<Entity>,
    pub has_military: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum BarbarianHordesPhase {
    #[default]
    FindLandingArea,
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
