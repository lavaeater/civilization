use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct SuperstitionState {
    pub phase: SuperstitionPhase,
    pub cities_to_reduce: usize,
    pub selected_cities: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum SuperstitionPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    ApplyEffects,
    Complete,
}

impl SuperstitionState {
    pub fn new() -> Self {
        Self {
            cities_to_reduce: 3,
            ..Default::default()
        }
    }
    
    pub fn with_mysticism(mut self) -> Self {
        self.cities_to_reduce = 2;
        self
    }
    
    pub fn with_deism(mut self) -> Self {
        self.cities_to_reduce = 1;
        self
    }
    
    pub fn with_enlightenment(mut self) -> Self {
        self.cities_to_reduce = 0;
        self
    }
    
    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.cities_to_reduce {
            self.selected_cities.push(city_area);
        }
    }
    
    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.cities_to_reduce
    }
}
