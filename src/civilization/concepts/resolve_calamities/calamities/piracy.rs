use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct PiracyState {
    pub phase: PiracyPhase,
    pub coastal_cities_to_replace: usize,
    pub selected_cities: Vec<Entity>,
    pub cities_to_replace: Vec<Entity>,
    pub beneficiary: Option<Entity>,
    pub immune_player: Option<Entity>,
    /// Cities selected from secondary victims (area entities). Rule 30.912.
    pub secondary_cities: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum PiracyPhase {
    #[default]
    DetermineBeneficiary,
    SelectCoastalCities,
    SelectSecondaryVictims,
    ApplyEffects,
    Complete,
}

impl PiracyState {
    pub fn new() -> Self {
        Self {
            coastal_cities_to_replace: 2,
            ..Default::default()
        }
    }

    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
}
