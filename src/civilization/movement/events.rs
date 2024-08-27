use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EndPlayerMovement;

#[derive(Event, Debug, Reflect)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub from_area: Entity,
    pub to_area: Entity,
    pub number_of_tokens: usize,
    pub player: Entity
}