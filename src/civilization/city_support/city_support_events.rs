use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EliminateCity {
    pub player: Entity,
    pub city: Entity,
    pub area_entity: Entity,
    pub conflict: bool
}

impl EliminateCity {
    pub fn new(player: Entity, city: Entity, area_entity: Entity, conflict:bool) -> Self {
        EliminateCity { player, city, area_entity, conflict }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct CheckPlayerCitySupport {
    pub player: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckCitySupportStatus;