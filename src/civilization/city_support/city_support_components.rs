use bevy::prelude::{Component, Reflect};

#[derive(Component, Debug, Reflect)]
pub struct HasTooManyCities {
    pub surplus_count: usize,
    pub needed_tokens: usize
}

#[derive(Component, Debug, Reflect)]
pub struct NeedsToCheckCitySupport;

