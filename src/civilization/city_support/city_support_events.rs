use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EliminateCity {
    pub city: Entity,
    pub area_entity: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckPlayerCitySupport {
    pub player: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckCitySupportStatus;