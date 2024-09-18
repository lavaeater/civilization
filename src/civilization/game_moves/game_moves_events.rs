use bevy::prelude::{Entity, Event};

#[derive(Event, Debug)]
pub struct RecalculatePlayerMoves {
    pub player: Entity,
}

impl RecalculatePlayerMoves {
    pub fn new(player: Entity) -> Self {
        RecalculatePlayerMoves {
            player
        }
    }
}