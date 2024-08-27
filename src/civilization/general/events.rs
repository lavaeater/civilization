use bevy::prelude::{Entity, Event};

#[derive(Event, Debug)]
pub struct MoveTokensFromStockToAreaCommand {
    pub area_entity: Entity,
    pub player_entity: Entity,
    pub number_of_tokens: usize,
}

#[derive(Event, Debug)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub from_area_population: Entity,
    pub to_area_population: Entity,
    pub tokens: Vec<Entity>,
}