use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct PiracyState {
    pub phase: PiracyPhase,
    pub coastal_cities_to_replace: usize,
    pub selected_cities: Vec<Entity>,
    pub beneficiary: Option<Entity>,
    pub immune_player: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum PiracyPhase {
    #[default]
    DetermineBeneficiary,
    SelectCoastalCities,
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
    
    pub fn with_beneficiary(mut self, beneficiary: Entity) -> Self {
        self.beneficiary = Some(beneficiary);
        self.phase = PiracyPhase::SelectCoastalCities;
        self
    }
    
    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
    
    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.coastal_cities_to_replace {
            self.selected_cities.push(city_area);
        }
    }
    
    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.coastal_cities_to_replace
    }
}
