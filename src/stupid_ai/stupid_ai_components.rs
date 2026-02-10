use bevy::prelude::*;

#[derive(Component, Debug, Reflect)]
pub struct StupidAi;

#[derive(Component, Debug, Reflect)]
pub struct IsHuman; //Not really, though

#[derive(Resource, Default)]
pub struct AiMoveQueue {
    pub pending: Vec<(Entity, f32)>,
}

impl AiMoveQueue {
    pub fn push(&mut self, player: Entity, delay: f32) {
        self.pending.push((player, delay));
    }
}
