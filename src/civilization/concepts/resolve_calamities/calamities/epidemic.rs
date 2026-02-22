use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct EpidemicState {
    pub phase: EpidemicPhase,
    pub primary_loss: i32,
    pub secondary_loss: i32,
    pub has_medicine: bool,
    pub immune_player: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum EpidemicPhase {
    #[default]
    ComputeEffects,
    ApplyPrimaryLoss,
    ApplySecondaryLosses,
    Complete,
}

impl EpidemicState {
    pub fn new() -> Self {
        Self {
            primary_loss: 16,
            secondary_loss: 8,
            ..Default::default()
        }
    }
    
    pub fn with_medicine(mut self) -> Self {
        self.has_medicine = true;
        self.primary_loss /= 2;
        self
    }
    
    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
}
