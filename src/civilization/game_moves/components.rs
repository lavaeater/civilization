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
    ModifyTradeOffer,
}

#[derive(Clone, Debug, Reflect)]
pub struct TradeMove {
    pub trade_move_type: TradeMoveType,
    pub trade_offer: Option<Entity>,
    pub request_commodities: Option<HashMap<Commodity, usize>>,
}

impl TradeMove {
    pub fn counter_trade_offer(trade_offer: Entity) -> Self {
        TradeMove::new(TradeMoveType::CounterTradeOffer, Some(trade_offer), None)
    }
    pub fn accept_trade_offer(trade_offer: Entity) -> Self {
        TradeMove::new(TradeMoveType::AcceptTradeOffer, Some(trade_offer), None)
    }
    pub fn decline_trade_offer(trade_offer: Entity) -> Self {
        TradeMove::new(TradeMoveType::DeclineTradeOffer, Some(trade_offer), None)
    }
    pub fn modify_trade_offer(trade_offer: Entity) -> Self {
        TradeMove::new(TradeMoveType::ModifyTradeOffer, Some(trade_offer), None)
    }
    
    pub fn open_trade_offer(request_commodities: HashMap<Commodity, usize>) -> Self {
        TradeMove::new(TradeMoveType::OpenTradeOffer, None, Some(request_commodities))
    }
    
    pub fn new(trade_move_type: TradeMoveType, trade_offer: Option<Entity>, request_commodities: Option<HashMap<Commodity, usize>>) -> Self {
        TradeMove {
            trade_move_type,
            trade_offer,
            request_commodities,
        }
    }
}