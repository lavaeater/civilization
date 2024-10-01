use bevy::prelude::{Entity, Event, Reflect};

#[derive(Event, Debug, Reflect)]
pub struct SelectStupidMove {
    pub player: Entity,
}

impl SelectStupidMove {
    pub fn new(player: Entity) -> Self {
        SelectStupidMove { player }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct StupidAiEvent {
    pub player: Entity,
}

impl StupidAiEvent {
    pub fn new(player: Entity) -> Self {
        StupidAiEvent { player }
    }
}