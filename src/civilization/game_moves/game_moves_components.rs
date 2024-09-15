use bevy::prelude::{Component, Entity};

#[derive(Component, Debug, Default)]
pub struct AvailableMoves {
    pub moves: Vec<Move>,
}

impl AvailableMoves {
    pub fn new(moves: Vec<Move>) -> Self {
        AvailableMoves {
            moves
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    PopulationExpansion(i32, Entity, usize),
}