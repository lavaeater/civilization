use bevy::prelude::{Entity, Message, Reflect};

#[derive(Message, Debug, Reflect)]
pub struct EndPlayerCityConstruction {
    pub player: Entity,
}

impl EndPlayerCityConstruction {
    pub fn new(player: Entity) -> Self {
        EndPlayerCityConstruction { player }
    }
}

#[derive(Message, Debug, Reflect)]
pub struct BuildCityCommand {
    pub player: Entity,
    pub area: Entity,
}

impl BuildCityCommand {
    pub fn new(player: Entity, area: Entity) -> Self {
        BuildCityCommand { player, area }
    }
}
