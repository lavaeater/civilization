use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EliminateCity {
    pub city: Entity,
    pub area_entity: Entity,
}

impl EliminateCity {
    pub fn new(city: Entity, area_entity: Entity) -> Self {
        EliminateCity { city, area_entity }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct CheckPlayerCitySupport {
    pub player: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckCitySupportStatus;