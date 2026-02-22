use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct FloodState {
    pub phase: FloodPhase,
    pub flood_plain_area: Option<Entity>,
    pub city_destroyed: bool,
    pub has_engineering: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum FloodPhase {
    #[default]
    FindFloodPlain,
    ApplyEffects,
    Complete,
}

impl FloodState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_flood_plain(mut self, area: Entity) -> Self {
        self.flood_plain_area = Some(area);
        self.phase = FloodPhase::ApplyEffects;
        self
    }
    
    pub fn with_engineering(mut self) -> Self {
        self.has_engineering = true;
        self
    }
}
