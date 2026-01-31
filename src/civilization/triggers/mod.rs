use crate::civilization::components::*;
use crate::civilization::functions::return_token_to_stock;
use bevy::prelude::{Commands, Add, Query, Sprite, Transform, On, Visibility};

pub fn on_add_return_token_to_stock(
    trigger: On<Add, ReturnTokenToStock>,
    token_query: Query<&Token>,
    mut player_query: Query<(&mut TokenStock, &mut PlayerAreas)>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.event().entity)
        .remove::<(ReturnTokenToStock, Sprite, Transform, Visibility)>();
    if let Ok(token) = token_query.get(trigger.event().entity) {
        if let Ok((mut token_stock, mut player_areas)) = player_query.get_mut(token.player()) {
            return_token_to_stock(trigger.event().entity, &mut token_stock, &mut player_areas);
        }
    }
}
