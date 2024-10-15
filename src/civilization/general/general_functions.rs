use crate::civilization::city_construction::city_construction_events::BuildCityCommand;
use crate::civilization::conflict::conflict_components::UnresolvedConflict;
use crate::civilization::general::general_components::{BuiltCity, CityTokenStock, PlayerAreas, PlayerCities, TokenStock};
use crate::civilization::general::prelude::{Population, ReturnTokenToStock};
use bevy::asset::Handle;
use bevy::math::Vec3;
use bevy::prelude::{default, Commands, Entity, Image, Mut, SpriteBundle, Transform};

pub fn move_from_stock_to_area(player: Entity, area: Entity, at_most_tokens: usize, population: &mut Population, token_stock: &mut TokenStock, player_areas: &mut PlayerAreas) {
    let tokens = token_stock.remove_at_most_n_tokens_from_stock(at_most_tokens).unwrap_or_default();
    population.add_tokens_to_area(player, tokens.clone());
    player_areas.add_tokens_to_area(area, tokens);
}

pub fn return_all_tokens_from_area_to_player(player: &Entity, area: &Entity, population: &mut Population, token_stock: &mut TokenStock, player_areas: &mut PlayerAreas) {
    let tokens = population.remove_all_tokens_for_player(player);
    token_stock.return_tokens_to_stock(tokens.clone());
    player_areas.remove_area(area);
}

pub fn return_all_tokens_from_area_to_players(population: &mut Population, mut commands: &mut Commands) {
    for player in population.players() {
        return_all_tokens_from_area_for_player(population, &player, &mut commands);
    }
}

pub fn return_all_tokens_from_area_for_player(population: &mut Population, player: &Entity, commands: &mut Commands) {
    for token in population.remove_all_tokens_for_player(player) {
        commands.entity(token).insert(ReturnTokenToStock);
    }
}

pub fn build_city_in_area(
    commands: &mut Commands,
    texture: Handle<Image>,
    build_city: &BuildCityCommand,
    city_stock: &mut Mut<CityTokenStock>,
    player_cities: &mut Mut<PlayerCities>,
    area_transform: &Transform,
) {
    if let Some(city_token) = city_stock.get_token_from_stock() {
        player_cities.build_city_in_area(build_city.area, city_token);
        commands.entity(build_city.area)
            .insert(BuiltCity {
                player: build_city.player,
                city: city_token,
            });
        commands.entity(city_token)
            .insert(SpriteBundle {
                texture,
                transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                    .with_translation(area_transform.translation),
                ..default()
            });
    }
}

pub fn return_token_to_stock(token: Entity, token_stock: &mut TokenStock, player_areas: &mut PlayerAreas) {
    token_stock.return_token_to_stock(token);
    player_areas.remove_token(token);
}

pub fn return_all_tokens_to_stock(population: &mut Population, commands: &mut Commands) {
    for token in population.remove_all_tokens() {
        commands.entity(token).insert(ReturnTokenToStock);
    }
}

pub fn return_all_but_n_tokens_to_stock_for_player(population: &mut Population, player: &Entity, n: usize, commands: &mut Commands) {
    for token in population.remove_all_but_n_tokens(player, n).unwrap_or_default() {
        commands.entity(token).insert(ReturnTokenToStock);
    }
}

pub fn remove_n_tokens_from_each_player(players: &Vec<Entity>, population: &mut Population, commands: &mut Commands, token_rounds: usize) {
    for player in players {
        for token in population.remove_tokens_from_area(player, token_rounds).unwrap_or_default() {
            commands.entity(token).insert(ReturnTokenToStock);
        }
    }
}

pub fn replace_city_with_tokens_for_conflict(
    commands: &mut Commands,
    area_entity: Entity,
    mut population: &mut Population,
    built_city: &BuiltCity,
    city_stock: &mut CityTokenStock,
    mut token_stock: &mut TokenStock,
    player_cities: &mut PlayerCities,
    mut player_areas: &mut PlayerAreas,
) {
    commands.entity(area_entity).remove::<BuiltCity>();
    player_cities.remove_city_from_area(area_entity);
    city_stock.return_token_to_stock(built_city.city);
    move_from_stock_to_area(built_city.player, area_entity, 6, &mut population, &mut token_stock, &mut player_areas);
    commands.entity(area_entity).insert(UnresolvedConflict);
}


