use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect, Default)]
pub struct Census {
    pub population: usize,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct HasPopulation;
