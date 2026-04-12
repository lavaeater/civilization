use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct CivilWarState {
    pub phase: CivilWarPhase,
    pub beneficiary: Option<Entity>,
    pub victim_selection_points: usize,
    pub beneficiary_selection_points: usize,
    pub victim_selected_units: Vec<Entity>,
    pub victim_selected_cities: Vec<Entity>,
    pub beneficiary_selected_units: Vec<Entity>,
    pub beneficiary_selected_cities: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum CivilWarPhase {
    #[default]
    DetermineBeneficiary,
    VictimSelectsUnits,
    BeneficiarySelectsUnits,
    TransferFaction,
    Complete,
}

impl CivilWarState {
    pub fn new() -> Self {
        Self {
            victim_selection_points: 15,
            beneficiary_selection_points: 20,
            ..Default::default()
        }
    }
    
    pub fn with_beneficiary(mut self, beneficiary: Entity) -> Self {
        self.beneficiary = Some(beneficiary);
        self.phase = CivilWarPhase::VictimSelectsUnits;
        self
    }
    
    pub fn apply_music_bonus(&mut self) {
        self.victim_selection_points += 5;
    }
    
    pub fn apply_drama_poetry_bonus(&mut self) {
        self.victim_selection_points += 5;
    }
    
    pub fn apply_democracy_bonus(&mut self) {
        self.victim_selection_points += 10;
    }
    
    pub fn victim_points_selected(&self) -> usize {
        self.victim_selected_units.len() + (self.victim_selected_cities.len() * 5)
    }
    
    pub fn beneficiary_points_selected(&self) -> usize {
        self.beneficiary_selected_units.len() + (self.beneficiary_selected_cities.len() * 5)
    }
    
    pub fn victim_selection_complete(&self) -> bool {
        self.victim_points_selected() >= self.victim_selection_points
    }
    
    pub fn beneficiary_selection_complete(&self) -> bool {
        self.beneficiary_points_selected() >= self.beneficiary_selection_points
    }
    
    pub fn units_to_transfer(&self) -> impl Iterator<Item = &Entity> {
        self.beneficiary_selected_units.iter()
            .chain(self.beneficiary_selected_cities.iter())
    }
}
