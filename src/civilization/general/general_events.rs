use bevy::prelude::{Entity, Event};

#[derive(Event, Debug)]
pub struct MoveTokensFromStockToAreaCommand {
    pub area_entity: Entity,
    pub player_entity: Entity,
    pub number_of_tokens: usize,
}

impl MoveTokensFromStockToAreaCommand {
    pub fn new(area_entity: Entity, player_entity: Entity, number_of_tokens: usize) -> Self {
        MoveTokensFromStockToAreaCommand {
            area_entity,
            player_entity,
            number_of_tokens,
        }
    }
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


