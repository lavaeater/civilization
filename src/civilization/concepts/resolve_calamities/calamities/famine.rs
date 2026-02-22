use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct FamineState {
    pub phase: FaminePhase,
    pub primary_loss: i32,
    pub secondary_total: i32,
    pub max_per_secondary: i32,
    pub grain_reduction: i32,
    pub secondary_allocations: Vec<(Entity, i32)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum FaminePhase {
    #[default]
    ComputeLosses,
    SelectSecondaryVictims,
    ApplyPrimaryLoss,
    ApplySecondaryLosses,
    Complete,
}

impl FamineState {
    pub fn new() -> Self {
        Self {
            primary_loss: 10,
            secondary_total: 20,
            max_per_secondary: 8,
            ..Default::default()
        }
    }
    
    pub fn with_grain_reduction(mut self, grain_count: usize, has_pottery: bool) -> Self {
        if has_pottery {
            self.grain_reduction = (grain_count * 4) as i32;
            self.primary_loss = (self.primary_loss - self.grain_reduction).max(0);
        }
        self
    }
    
    pub fn add_secondary_allocation(&mut self, victim: Entity, points: i32) {
        let clamped = points.min(self.max_per_secondary);
        self.secondary_allocations.push((victim, clamped));
    }
    
    pub fn remaining_secondary_points(&self) -> i32 {
        let allocated: i32 = self.secondary_allocations.iter().map(|(_, p)| *p).sum();
        self.secondary_total - allocated
    }
}
