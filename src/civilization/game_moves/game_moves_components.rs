use bevy::prelude::{Component, Entity};
use bevy::utils::HashMap;

#[derive(Component, Debug, Default)]
pub struct AvailableMoves {
    pub moves: HashMap<usize, Move>,
}

impl AvailableMoves {
    pub fn new(moves: HashMap<usize, Move>) -> Self {
        AvailableMoves {
            moves
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    PopulationExpansion(Entity, usize),
}