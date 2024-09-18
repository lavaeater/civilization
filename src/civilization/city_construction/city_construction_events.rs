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
pub struct BuildCityCommand {
    pub player: Entity,
    pub area: Entity,
}

impl BuildCityCommand {
    pub fn new(player: Entity, area: Entity) -> Self {
        BuildCityCommand {
            player,
            area
        }
    }
}