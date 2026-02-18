use bevy::prelude::{Entity, Message};

#[derive(Message)]
pub struct PlayerDoneAcquiringCivilizationCards(pub Entity);