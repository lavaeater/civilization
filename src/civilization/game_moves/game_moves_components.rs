use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::HashMap;

#[derive(Component, Debug, Default, Reflect)]
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

#[derive(Debug, Clone, Copy, Reflect)]
pub enum Move {
    PopulationExpansion(PopExpMove),
    Movement(MovementMove),
    AttackArea(MovementMove),
    AttackCity(MovementMove),
    EndMovement,
    CityConstruction(BuildCityMove),
    EndCityConstruction,
    EliminateCity(EliminateCityMove),
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct EliminateCityMove {
    pub player: Entity,
    pub area: Entity,
    pub city: Entity,
    pub tokens_gained: usize,
    pub tokens_needed: usize,
}

impl EliminateCityMove {
    pub fn new(player: Entity, area: Entity, city: Entity, tokens_gained: usize, tokens_needed: usize) -> Self {
        EliminateCityMove {
            player,
            area,
            city,
            tokens_gained,
            tokens_needed
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct BuildCityMove {
    pub target: Entity,
    pub player: Entity,
}

impl BuildCityMove {
    pub fn new(target: Entity, player: Entity) -> Self {
        BuildCityMove {
            target,
            player,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct MovementMove {
    pub source: Entity,
    pub target: Entity,
    pub player: Entity,
    pub max_tokens: usize,
}

impl MovementMove {
    pub fn new(source: Entity, target: Entity, player: Entity, max_tokens: usize) -> Self {
        MovementMove {
            source,
            target,
            player,
            max_tokens,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct PopExpMove {
    pub area: Entity,
    pub max_tokens: usize,
}

impl PopExpMove {
    pub fn new(area: Entity, max_tokens: usize) -> Self {
        PopExpMove {
            area,
            max_tokens,
        }
    }
}