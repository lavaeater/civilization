use bevy::prelude::{Entity, Message, Reflect};

#[derive(Message, Debug, Reflect)]
pub struct SelectStupidMove {
    pub player: Entity,
}

impl SelectStupidMove {
    pub fn new(player: Entity) -> Self {
        SelectStupidMove { player }
    }
}

#[derive(Message, Debug, Reflect)]
pub struct StupidAiMessage {
    pub player: Entity,
}

impl StupidAiMessage {
    pub fn new(player: Entity) -> Self {
        StupidAiMessage { player }
    }
}
