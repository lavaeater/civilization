use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::TradeCard;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Component, Entity, Reflect};
use std::fmt::Display;

#[derive(Component, Debug, Default, Reflect)]
pub struct AvailableMoves {
    pub moves: HashMap<usize, Move>,
}

impl AvailableMoves {
    pub fn new(moves: HashMap<usize, Move>) -> Self {
        AvailableMoves { moves }
    }
}

#[derive(Debug, Clone, Reflect)]
pub enum Move {
    PopulationExpansion(PopExpMove),
    Movement(MovementMove),
    AttackArea(MovementMove),
    AttackCity(MovementMove),
    EndMovement,
    CityConstruction(BuildCityMove),
    EndCityConstruction,
    EliminateCity(EliminateCityMove),
    Trade(TradeMove),
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
#[derive(Clone, Debug, Reflect)]
pub struct EliminateCityMove {
    pub player: Entity,
    pub area: Entity,
    pub city: Entity,
    pub tokens_gained: usize,
    pub tokens_needed: usize,
}

impl EliminateCityMove {
    pub fn new(
        player: Entity,
        area: Entity,
        city: Entity,
        tokens_gained: usize,
        tokens_needed: usize,
    ) -> Self {
        EliminateCityMove {
            player,
            area,
            city,
            tokens_gained,
            tokens_needed,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct BuildCityMove {
    pub target: Entity,
    pub player: Entity,
}

impl BuildCityMove {
    pub fn new(target: Entity, player: Entity) -> Self {
        BuildCityMove { target, player }
    }
}

#[derive(Clone, Debug, Reflect)]
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

#[derive(Clone, Debug, Reflect)]
pub struct PopExpMove {
    pub area: Entity,
    pub max_tokens: usize,
}

impl PopExpMove {
    pub fn new(area: Entity, max_tokens: usize) -> Self {
        PopExpMove { area, max_tokens }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum TradeMove {
    ProposeTrade(Entity, HashMap<TradeCard, usize>),
    AcceptOrDeclineTrade(Entity),
    AutoDeclineTrade(Entity),
    StopTrading,
    SettleTrade(Entity),
}
