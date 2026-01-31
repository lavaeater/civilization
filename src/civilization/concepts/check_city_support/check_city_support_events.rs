use bevy::prelude::{Entity, Message, Reflect};

#[derive(Message, Debug, Reflect)]
pub struct EliminateCity {
    pub player: Entity,
    pub city: Entity,
    pub area_entity: Entity,
    pub is_conflict: bool,
}

impl EliminateCity {
    pub fn new(player: Entity, city: Entity, area_entity: Entity, conflict: bool) -> Self {
        EliminateCity {
            player,
            city,
            area_entity,
            is_conflict: conflict,
        }
    }
}

#[derive(Message, Debug, Reflect)]
pub struct CheckPlayerCitySupport {
    pub player: Entity,
}

#[derive(Message, Debug, Reflect)]
pub struct CheckCitySupportStatus;
