use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::HashMap;
use crate::civilization::concepts::trade_cards::enums::Commodity;

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

#[derive(Clone, Debug, Reflect)]
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

#[derive(Clone, Debug, Reflect)]
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
        PopExpMove {
            area,
            max_tokens,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum TradeMoveType {
    OpenTradeOffer,
    AcceptTradeOffer,
    DeclineTradeOffer,
    CounterTradeOffer,
}

#[derive(Clone, Debug, Reflect)]
pub struct TradeMove {
    pub trade_move_type: TradeMoveType,
    pub trade_offer: Option<Entity>,
    pub request_commodities: Option<HashMap<Commodity, usize>>,
    pub offer_commodities: Option<HashMap<Commodity, usize>>,
}