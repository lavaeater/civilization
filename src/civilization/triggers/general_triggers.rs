use crate::civilization::components::prelude::*;
use crate::civilization::functions::prelude::return_token_to_stock;
use bevy::prelude::{Commands, Component, OnAdd, Query, Sprite, Trigger};

#[derive(Debug, Component)]
pub struct ReturnTokenToStock;

pub fn on_add_return_token_to_stock(
    trigger: Trigger<OnAdd, ReturnTokenToStock>,
    token_query: Query<&Token>,
    mut player_query: Query<(&mut TokenStock, &mut PlayerAreas)>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity())
        .remove::<ReturnTokenToStock>()
        .remove_with_requires::<Sprite>()
    ;
    if let Ok(token) = token_query.get(trigger.entity()) {
        if let Ok ((mut token_stock, mut player_areas)) = player_query.get_mut(token.player()) {
            return_token_to_stock(trigger.entity(), &mut token_stock, &mut player_areas);
        }
    }
}