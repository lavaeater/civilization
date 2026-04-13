use bevy::prelude::{Entity, Reflect};

/// State for resolving Flood (rule 30.51).
///
/// The flood plain with the most primary victim unit points is chosen.
/// Primary victim loses max 17 pts (7 with Engineering) from that flood plain.
/// Secondary victims on the same flood plain collectively lose 10 pts (auto-distributed).
/// If no flood plain has victim units: one coastal city is eliminated (reduced with Engineering).
#[derive(Debug, Clone, Default, Reflect)]
pub struct FloodState {
    pub phase: FloodPhase,
    /// The flood plain area where the flood occurs.
    pub flood_plain_area: Option<Entity>,
    /// How many unit points the primary victim loses (capped at 17, or 7 with Engineering).
    pub primary_max_loss: i32,
    pub has_engineering: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum FloodPhase {
    #[default]
    FindFloodPlain,
    ApplyPrimaryLoss,
    ApplySecondaryLoss,
    FallbackCoastalCity,
    Complete,
}

impl FloodState {
    pub fn new() -> Self {
        Self {
            primary_max_loss: 17,
            ..Default::default()
        }
    }

    pub fn with_engineering(mut self) -> Self {
        self.has_engineering = true;
        self.primary_max_loss = 7;
        self
    }
}
