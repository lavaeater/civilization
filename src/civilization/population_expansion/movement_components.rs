use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug)]
pub struct ExpandManually;

#[derive(Component, Debug, Reflect)]
pub struct ExpandAutomatically {
    pub required_tokens: usize,
}

#[derive(Component, Debug)]
pub struct NeedsExpansion;