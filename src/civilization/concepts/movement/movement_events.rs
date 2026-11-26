use bevy::prelude::{Entity, Message, Reflect};

/// Command: sail one of the player's ships from `source_area` to
/// `target_area` (any coast within its remaining range). Before it sails,
/// `drop` of the tokens aboard step off at `source_area`, then up to
/// `number_of_tokens` unmoved tokens there step aboard. Everything aboard
/// sails with it; with `unload_on_arrival` they all step off at the target.
#[derive(Message, Debug, Reflect)]
pub struct ShipFerryCommand {
    pub source_area: Entity,
    pub target_area: Entity,
    pub number_of_tokens: usize,
    pub player: Entity,
    /// The ship to sail; `None` picks one in `source_area`.
    pub ship: Option<Entity>,
    pub drop: usize,
    pub unload_on_arrival: bool,
}

impl ShipFerryCommand {
    /// A one-trip ferry: load, sail, unload everyone at the target.
    pub fn new(
        source_area: Entity,
        target_area: Entity,
        number_of_tokens: usize,
        player: Entity,
    ) -> Self {
        Self {
            source_area,
            target_area,
            number_of_tokens,
            player,
            ship: None,
            drop: 0,
            unload_on_arrival: true,
        }
    }

    /// One leg of a longer trip: drop `drop`, load `load`, sail on with
    /// everyone still aboard.
    pub fn leg(
        source_area: Entity,
        target_area: Entity,
        drop: usize,
        load: usize,
        player: Entity,
    ) -> Self {
        Self {
            drop,
            unload_on_arrival: false,
            ..Self::new(source_area, target_area, load, player)
        }
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
