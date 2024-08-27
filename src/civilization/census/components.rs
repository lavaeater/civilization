use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct Census {
    pub population: usize,
}

#[derive(Component, Debug, Reflect)]
pub struct HasPopulation;