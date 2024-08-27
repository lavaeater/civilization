use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EndPlayerMovement;

#[derive(Event, Debug)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub from_area_population: Entity,
    pub to_area_population: Entity,
    pub tokens: Vec<Entity>,
}