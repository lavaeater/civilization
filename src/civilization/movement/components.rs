use bevy::prelude::{Component, Entity, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct PerformingMovement;

#[derive(Component, Debug, Reflect)]
pub struct TokenCanMove;

#[derive(Component, Debug, Reflect)]
pub struct MoveableTokens {
    pub tokens: Vec<Entity>,
    pub targets: Vec<Entity>,
}

#[derive(Component, Debug, Reflect)]
pub struct NeedsTocalculateMoves;
