use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EliminateCity {
    pub player: Entity,
    pub city: Entity,
    pub area_entity: Entity,
}

impl EliminateCity {
    pub fn new(player: Entity, city: Entity, area_entity: Entity) -> Self {
        EliminateCity { player, city, area_entity }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct CheckPlayerCitySupport {
    pub player: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckCitySupportStatus;