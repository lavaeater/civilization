use bevy::prelude::{Component, Entity, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct CityBuildTargets {
    pub targets: Vec<Entity>,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct IsBuilding;
