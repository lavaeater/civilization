use crate::civilization::general::prelude::*;
use bevy::core::Name;
use bevy::prelude::{Commands, Component, Entity, OnAdd, Query, Trigger};

#[derive(Debug, Component)]
pub struct ReturnTokenToStock {
    pub token: Entity,
}

pub fn on_add_return_token_to_stock(
    trigger: Trigger<OnAdd, ReturnTokenToStock>,
    mut areas: Query<(Entity, &Name, &mut Population)>,
    mut commands: Commands,
) {
    
}