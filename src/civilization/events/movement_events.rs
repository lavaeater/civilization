use bevy::prelude::{Entity, Event, Reflect};
use bevy::utils::HashMap;
use crate::civilization::concepts::trade_cards::enums::Commodity;

#[derive(Event, Debug, Reflect)]
pub struct PlayerMovementEnded {
    pub player: Entity
}

impl PlayerMovementEnded {
    pub fn new(player: Entity) -> Self {
        PlayerMovementEnded {
            player
        }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct NextPlayerStarted;

#[derive(Event, Debug, Reflect)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub source_area: Entity,
    pub target_area: Entity,
    pub number_of_tokens: usize,
    pub player: Entity
}

impl MoveTokenFromAreaToAreaCommand {
    pub fn new(source_area: Entity, target_area: Entity, number_of_tokens: usize, player:Entity) -> Self {
        MoveTokenFromAreaToAreaCommand {
            source_area,
            target_area,
            number_of_tokens,
            player            
        }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct SendCardsToPlayerCommand {
    pub source_player: Entity,
    pub target_player: Entity,
    pub cards: HashMap<Commodity, usize>,
}

impl SendCardsToPlayerCommand {
    pub fn new(source_player: Entity, target_player: Entity, cards: HashMap<Commodity, usize>) -> Self {
        SendCardsToPlayerCommand {
            source_player,
            target_player,
            cards,
        }
    }
}