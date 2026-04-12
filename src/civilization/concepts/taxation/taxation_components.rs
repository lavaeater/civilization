use bevy::prelude::{Component, Entity, Reflect, ReflectComponent};

/// Marks a player as needing to pay taxes this turn. Removed once taxes are paid
/// (or revolts are resolved).
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct NeedsToPayTaxes {
    /// Number of tokens the player must transfer from stock to treasury.
    pub tokens_owed: usize,
}

impl NeedsToPayTaxes {
    pub fn new(tokens_owed: usize) -> Self {
        Self { tokens_owed }
    }
}

/// Marks a city that is revolting this turn. Carries the entity of the player
/// that owns the revolting city, so the revolt resolution system can find the
/// correct beneficiary.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CityInRevolt {
    pub original_owner: Entity,
}

impl CityInRevolt {
    pub fn new(original_owner: Entity) -> Self {
        Self { original_owner }
    }
}
