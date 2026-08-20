use crate::civilization::components::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::ReturnCityToStock;
use crate::civilization::functions::return_token_to_stock;
use bevy::prelude::{Add, Commands, Entity, On, Query, Sprite, Transform, Visibility, warn};

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
            warn!(
                "[RETURN_TOKEN] Failed to get player components for token {:?}, player {:?}",
                entity,
                token.player()
            );
        }
    } else {
        warn!(
            "[RETURN_TOKEN] Failed to get Token component for entity {:?}",
            entity
        );
    }
}

/// Retires a city token: strips what makes it visible on the map and hands it
/// back to its owner's stock.
///
/// City removal used to update `CityTokenStock`/`PlayerCities` but leave the
/// token's `Sprite` and `Transform` in place, so a destroyed or reduced city
/// kept its icon sitting on the board -- the replacement population tokens
/// were placed underneath it and the whole calamity read, to the player, as
/// having done nothing at all.
pub fn on_add_return_city_to_stock(
    trigger: On<Add, ReturnCityToStock>,
    city_token_query: Query<&CityToken>,
    mut owner_query: Query<(&mut CityTokenStock, &mut PlayerCities)>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    retire_city_token_visuals(&mut commands, entity);
    commands.entity(entity).remove::<ReturnCityToStock>();

    let Ok(city_token) = city_token_query.get(entity) else {
        warn!("[RETURN_CITY] {entity:?} is not a city token");
        return;
    };
    let Ok((mut city_stock, mut player_cities)) = owner_query.get_mut(city_token.player) else {
        warn!(
            "[RETURN_CITY] no stock/cities for {:?}, owner of city {entity:?}",
            city_token.player
        );
        return;
    };
    player_cities.remove_city_by_token(entity);
    city_stock.return_token_to_stock(entity);
}

/// Removes the rendering components from a city token that has left the board.
/// Call this from any path that returns a city token to stock itself.
pub fn retire_city_token_visuals(commands: &mut Commands, city_token: Entity) {
    commands
        .entity(city_token)
        .remove::<(Sprite, Transform, Visibility)>();
}
