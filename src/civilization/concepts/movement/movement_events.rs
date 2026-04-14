use bevy::prelude::{Entity, Message, Reflect};

/// Command: use a ship in `source_area` to ferry `number_of_tokens` unmoved tokens
/// to `target_area` via a sea passage. Also moves the ship entity to the target.
#[derive(Message, Debug, Reflect)]
pub struct ShipFerryCommand {
    pub source_area: Entity,
    pub target_area: Entity,
    pub number_of_tokens: usize,
    pub player: Entity,
}

impl ShipFerryCommand {
    pub fn new(source_area: Entity, target_area: Entity, number_of_tokens: usize, player: Entity) -> Self {
        Self { source_area, target_area, number_of_tokens, player }
    }
}

#[derive(Message, Debug, Reflect)]
pub struct PlayerMovementEnded {
    pub player: Entity,
}

impl PlayerMovementEnded {
    pub fn new(player: Entity) -> Self {
        PlayerMovementEnded { player }
    }
}

#[derive(Message, Debug, Reflect)]
pub struct NextPlayerStarted;

#[derive(Message, Debug, Reflect)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub source_area: Entity,
    pub target_area: Entity,
    pub number_of_tokens: usize,
    pub player: Entity,
}

impl MoveTokenFromAreaToAreaCommand {
    pub fn new(
        source_area: Entity,
        target_area: Entity,
        number_of_tokens: usize,
        player: Entity,
    ) -> Self {
        MoveTokenFromAreaToAreaCommand {
            source_area,
            target_area,
            number_of_tokens,
            player,
        }
    }
}
