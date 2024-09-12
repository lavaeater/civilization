use bevy::prelude::{Entity, Event};

#[derive(Event, Debug)]
pub struct MoveTokensFromStockToAreaCommand {
    pub area_entity: Entity,
    pub player_entity: Entity,
    pub number_of_tokens: usize,
}

#[derive(Event, Debug)]
pub struct ReturnTokenToStock {
    pub token_entity: Entity,
}

impl ReturnTokenToStock{
    pub fn new(token_entity: Entity) -> Self {
        ReturnTokenToStock { token_entity }
    }
}


