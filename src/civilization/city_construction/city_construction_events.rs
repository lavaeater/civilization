use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct EndPlayerCityConstruction {
    pub player: Entity,
}

impl EndPlayerCityConstruction {
    pub fn new(player: Entity) -> Self {
        EndPlayerCityConstruction {
            player
        }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct BuildCity {
    pub player: Entity,
    pub area: Entity,
}

impl BuildCity {
    pub fn new(player: Entity, area: Entity) -> Self {
        BuildCity {
            player,
            area
        }
    }
}