use bevy::prelude::{Component, Reflect, ReflectComponent};

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Census {
    pub population: usize,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct HasPopulation;
