use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct BarbarianHordesState {
    pub phase: BarbarianHordesPhase,
    pub unit_points_to_lose: i32,
    pub areas_attacked: Vec<Entity>,
    pub has_military: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum BarbarianHordesPhase {
    #[default]
    ComputeEffects,
    SelectAreas,
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
