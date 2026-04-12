use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct IconoclasmHeresyState {
    pub phase: IconoclasmHeresyPhase,
    pub primary_loss: i32,
    pub secondary_loss: i32,
    pub has_theology: bool,
    pub has_philosophy: bool,
    pub immune_player: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum IconoclasmHeresyPhase {
    #[default]
    ComputeEffects,
    SelectSecondaryVictims,
    ApplyPrimaryLoss,
    ApplySecondaryLosses,
    Complete,
}

impl IconoclasmHeresyState {
    pub fn new() -> Self {
        Self {
            primary_loss: 12,
            secondary_loss: 6,
            ..Default::default()
        }
    }
    
    pub fn with_theology(mut self) -> Self {
        self.has_theology = true;
        self.primary_loss = (self.primary_loss - 4).max(0);
        self
    }
    
    pub fn with_philosophy(mut self) -> Self {
        self.has_philosophy = true;
        self.primary_loss = (self.primary_loss - 2).max(0);
        self
    }
    
    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
}
