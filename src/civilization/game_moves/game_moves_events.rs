use bevy::prelude::{Entity, Message};

#[derive(Message, Debug)]
pub struct RecalculatePlayerMoves {
    pub player: Entity,
}

impl RecalculatePlayerMoves {
    pub fn new(player: Entity) -> Self {
        RecalculatePlayerMoves { player }
    }
}
