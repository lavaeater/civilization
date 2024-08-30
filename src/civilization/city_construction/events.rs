use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EndCityConstructionActivity;

#[derive(Event, Debug, Reflect)]
pub struct BuildCity {
    pub player: Entity,
    pub area: Entity,
}