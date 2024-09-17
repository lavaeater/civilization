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
    PopulationExpansion(PopExpMove),
    Movement(MovementMove),
}

#[derive(Clone, Copy, Debug)]
pub struct MovementMove {
    pub source: Entity,
    pub target: Entity,
    pub player: Entity,
    pub max_tokens: usize
}

#[derive(Clone, Copy, Debug)]
pub struct PopExpMove {
    pub area: Entity,
    pub max_tokens: usize,
}

impl PopExpMove {
    pub fn new(area: Entity, max_tokens: usize) -> Self {
        PopExpMove {
            area,
            max_tokens
        }
    }
}