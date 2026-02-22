use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct CivilDisorderState {
    pub phase: CivilDisorderPhase,
    pub cities_to_reduce: usize,
    pub selected_cities: Vec<Entity>,
    pub has_democracy: bool,
    pub has_law: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum CivilDisorderPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    ApplyEffects,
    Complete,
}

impl CivilDisorderState {
    pub fn new(city_count: usize) -> Self {
        Self {
            cities_to_reduce: city_count,
            ..Default::default()
        }
    }
    
    pub fn with_democracy(mut self) -> Self {
        self.has_democracy = true;
        self.cities_to_reduce = 0;
        self
    }
    
    pub fn with_law(mut self) -> Self {
        self.has_law = true;
        self.cities_to_reduce = self.cities_to_reduce.saturating_sub(1);
        self
    }
    
    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.cities_to_reduce {
            self.selected_cities.push(city_area);
        }
    }
    
    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.cities_to_reduce || self.has_democracy
    }
}
