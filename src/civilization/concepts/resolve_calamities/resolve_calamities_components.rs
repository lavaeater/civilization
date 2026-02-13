use bevy::prelude::{Component, Entity, Reflect, ReflectComponent};
use crate::civilization::TradeCard;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct NeedsCalamityResolution;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CalamityVictim {
    pub calamity: TradeCard,
    pub traded_by: Option<Entity>,
}

impl CalamityVictim {
    pub fn new(calamity: TradeCard, traded_by: Option<Entity>) -> Self {
        Self { calamity, traded_by }
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PendingCalamities {
    pub calamities: Vec<(TradeCard, Option<Entity>)>,
}

impl PendingCalamities {
    pub fn new(calamities: Vec<(TradeCard, Option<Entity>)>) -> Self {
        Self { calamities }
    }
    
    pub fn count(&self) -> usize {
        self.calamities.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.calamities.is_empty()
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ResolvingVolcanoEarthquake {
    pub is_volcano: bool,
    pub target_areas: Vec<Entity>,
}
