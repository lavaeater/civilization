use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct SlaveRevoltState {
    pub phase: SlaveRevoltPhase,
    pub cities_to_reduce: usize,
    pub selected_cities: Vec<Entity>,
    pub has_enlightenment: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum SlaveRevoltPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    ApplyEffects,
    Complete,
}

impl SlaveRevoltState {
    pub fn new(city_count: usize) -> Self {
        Self {
            cities_to_reduce: city_count,
            ..Default::default()
        }
    }
    
    pub fn with_enlightenment(mut self) -> Self {
        self.has_enlightenment = true;
        self.cities_to_reduce = 0;
        self
    }
    
    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.cities_to_reduce {
            self.selected_cities.push(city_area);
        }
    }
    
    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.cities_to_reduce || self.has_enlightenment
    }
}
