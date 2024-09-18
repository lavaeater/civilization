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
    EndMovement,
    CityConstruction(BuildCityMove),
    EndCityConstruction,
}

#[derive(Clone, Copy, Debug)]
pub struct BuildCityMove {
    pub target: Entity,
    pub player: Entity
}

impl BuildCityMove {
    pub fn new(target: Entity, player: Entity) -> Self {
        BuildCityMove {
            target,
            player
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MovementMove {
    pub source: Entity,
    pub target: Entity,
    pub player: Entity,
    pub max_tokens: usize
}

impl MovementMove {
    pub fn new(source: Entity, target: Entity, player: Entity, max_tokens: usize) -> Self {
        MovementMove {
            source,
            target,
            player,
            max_tokens
        }
    }
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