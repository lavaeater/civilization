use bevy::prelude::{Entity, Reflect};

/// State for resolving Epidemic (rule 30.61).
///
/// Default primary loss: 16 unit points (30.611).
/// Default secondary loss: 25 unit points total, ≤10 per player (30.611).
///
/// Modifiers (cumulative, 30.615):
/// - Medicine     : primary loss −8 (30.613); secondary loss −5 for Medicine holders
/// - Road Building: primary/secondary loss +5 (30.614)
#[derive(Debug, Clone, Default, Reflect)]
pub struct EpidemicState {
    pub phase: EpidemicPhase,
    pub primary_loss: i32,
    /// Total unit points to distribute among secondary victims.
    pub secondary_loss: i32,
    pub has_medicine: bool,
    pub has_road_building: bool,
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
            secondary_loss: 25,
            ..Default::default()
        }
    }

    /// Medicine: primary victim loses 8 fewer unit points (30.613).
    /// Secondary victims who hold Medicine lose 5 fewer — handled during secondary resolution.
    pub fn with_medicine(mut self) -> Self {
        self.has_medicine = true;
        self.primary_loss = (self.primary_loss - 8).max(0);
        self
    }

    /// Road Building: primary (and secondary) victim loses 5 additional unit points (30.614).
    pub fn with_road_building(mut self) -> Self {
        self.has_road_building = true;
        self.primary_loss += 5;
        self
    }

    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
}
