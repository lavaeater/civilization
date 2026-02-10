use crate::civilization::components::*;
use crate::civilization::functions::return_token_to_stock;
use bevy::prelude::{Commands, Add, Query, Sprite, Transform, On, Visibility, warn};

pub fn on_add_return_token_to_stock(
    trigger: On<Add, ReturnTokenToStock>,
    token_query: Query<&Token>,
    mut player_query: Query<(&mut TokenStock, &mut PlayerAreas)>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    commands
        .entity(entity)
        .remove::<(ReturnTokenToStock, Sprite, Transform, Visibility)>();
    
    if let Ok(token) = token_query.get(entity) {
        if let Ok((mut token_stock, mut player_areas)) = player_query.get_mut(token.player()) {
            return_token_to_stock(entity, &mut token_stock, &mut player_areas);
        } else {
            warn!("[RETURN_TOKEN] Failed to get player components for token {:?}, player {:?}", entity, token.player());
        }
    } else {
        warn!("[RETURN_TOKEN] Failed to get Token component for entity {:?}", entity);
    }
}
