use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug)]
pub struct ExpandManually {
    pub required_tokens: usize,
}

impl ExpandManually {
    pub fn new(required_tokens: usize) -> Self {
        ExpandManually {
            required_tokens
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct ExpandAutomatically {
    pub required_tokens: usize,
}

impl ExpandAutomatically{
    pub fn new(required_tokens: usize) -> Self {
        ExpandAutomatically {
            required_tokens
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct NeedsExpansion;