use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct PerformingMovement;

#[derive(Component, Debug, Reflect)]
pub struct HasJustMoved;

#[derive(Component, Debug, Reflect)]
pub struct TokenHasMoved;
