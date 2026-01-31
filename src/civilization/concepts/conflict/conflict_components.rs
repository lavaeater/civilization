use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct UnresolvedConflict;

#[derive(Component, Debug, Reflect)]
pub struct UnresolvedCityConflict;
