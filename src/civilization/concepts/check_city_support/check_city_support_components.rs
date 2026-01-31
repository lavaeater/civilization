use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct HasTooManyCities {
    pub surplus_count: usize,
    pub needed_tokens: usize,
}

impl HasTooManyCities {
    pub fn new(surplus_count: usize, needed_tokens: usize) -> Self {
        HasTooManyCities {
            surplus_count,
            needed_tokens,
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct NeedsToCheckCitySupport;
