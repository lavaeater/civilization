use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct TreacheryState {
    pub phase: TreacheryPhase,
    pub city_to_replace: Option<Entity>,
    pub beneficiary: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum TreacheryPhase {
    #[default]
    SelectCity,
    ApplyEffects,
    Complete,
}

impl TreacheryState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_trade(city: Entity, beneficiary: Entity) -> Self {
        Self {
            phase: TreacheryPhase::ApplyEffects,
            city_to_replace: Some(city),
            beneficiary: Some(beneficiary),
        }
    }
    
    pub fn without_trade(city: Entity) -> Self {
        Self {
            phase: TreacheryPhase::ApplyEffects,
            city_to_replace: Some(city),
            beneficiary: None,
        }
    }
}
