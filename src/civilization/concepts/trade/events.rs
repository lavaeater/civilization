use bevy::prelude::{Event, Reflect};

#[derive(Event, Reflect, Default, Clone, PartialEq)]
pub struct StartTrading;