use bevy::prelude::{Entity, Reflect, Resource};

#[derive(Resource, Debug, Reflect, Default)]
pub struct GameInfoAndStuff {
    pub census_order: Vec<Entity>,
    pub left_to_move: Vec<Entity>,
    pub round: usize,
}
