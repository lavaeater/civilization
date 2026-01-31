use bevy::prelude::{Entity, Message};

#[derive(Message, Debug)]
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
