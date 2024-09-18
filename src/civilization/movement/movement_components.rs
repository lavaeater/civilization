use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect, Default)]
pub struct PerformingMovement;

#[derive(Component, Debug, Reflect, Default)]
pub struct TokenHasMoved;
