use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct PlayerMovementEnded;

#[derive(Event, Debug, Reflect)]
pub struct NextPlayerStarted;

#[derive(Event, Debug, Reflect)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub number_of_tokens: usize,
    pub player: Entity
}