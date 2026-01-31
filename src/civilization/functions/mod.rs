use crate::civilization::components::*;
use bevy::asset::Handle;
use bevy::math::Vec3;
use bevy::prelude::{default, Commands, Entity, Image, Mut, Sprite, Transform};
use crate::civilization::components::population::Population;
use crate::civilization::concepts::city_construction::city_construction_events::BuildCityCommand;

pub fn move_from_stock_to_area(
    player: Entity,
    area: Entity,
    at_most_tokens: usize,
    population: &mut Population,
    token_stock: &mut TokenStock,
    player_areas: &mut PlayerAreas,
) {
    let tokens = token_stock
        .remove_at_most_n_tokens_from_stock(at_most_tokens)
        .unwrap_or_default();
    population.add_tokens_to_area(player, tokens.clone());
    player_areas.add_tokens_to_area(area, tokens);
}

pub fn return_all_tokens_from_area_to_player(
    player: &Entity,
    area: &Entity,
    population: &mut Population,
    token_stock: &mut TokenStock,
    player_areas: &mut PlayerAreas,
) {
    let tokens = population.remove_all_tokens_for_player(player);
    token_stock.return_tokens_to_stock(tokens.clone());
    player_areas.remove_area(area);
}

pub fn return_all_tokens_from_area_to_players(
    population: &mut Population,
    mut commands: &mut Commands,
) {
    for player in population.players() {
        return_all_tokens_from_area_for_player(population, &player, &mut commands);
    }
}

pub fn return_all_tokens_from_area_for_player(
    population: &mut Population,
    player: &Entity,
    commands: &mut Commands,
) {
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
        commands.entity(build_city.area).insert(BuiltCity {
            player: build_city.player,
            city: city_token,
        });
        commands.entity(city_token).insert((
            Sprite {
                image: texture,
                ..default()
            },
            Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                .with_translation(area_transform.translation),
        ));
    }
}

pub fn return_token_to_stock(
    token: Entity,
    token_stock: &mut TokenStock,
    player_areas: &mut PlayerAreas,
) {
    token_stock.return_token_to_stock(token);
    player_areas.remove_token(token);
}

pub fn return_all_tokens_to_stock(population: &mut Population, commands: &mut Commands) {
    for token in population.remove_all_tokens() {
        commands.entity(token).insert(ReturnTokenToStock);
    }
}

pub fn return_all_but_n_tokens_to_stock_for_player(
    population: &mut Population,
    player: &Entity,
    n: usize,
    commands: &mut Commands,
) {
    for token in population
        .remove_all_but_n_tokens(player, n)
        .unwrap_or_default()
    {
        commands.entity(token).insert(ReturnTokenToStock);
    }
}

pub fn remove_n_tokens_from_each_player(
    players: &Vec<Entity>,
    population: &mut Population,
    commands: &mut Commands,
    token_rounds: usize,
) {
    for player in players {
        for token in population
            .remove_tokens_from_area(player, token_rounds)
            .unwrap_or_default()
        {
            commands.entity(token).insert(ReturnTokenToStock);
        }
    }
}

pub fn replace_city_with_tokens_for_conflict(
    area_entity: Entity,
    population: &mut Population,
    built_city: &BuiltCity,
    city_stock: &mut CityTokenStock,
    token_stock: &mut TokenStock,
    player_cities: &mut PlayerCities,
    player_areas: &mut PlayerAreas,
) {
    player_cities.remove_city_from_area(area_entity);
    city_stock.return_token_to_stock(built_city.city);
    move_from_stock_to_area(
        built_city.player,
        area_entity,
        6,
        population,
        token_stock,
        player_areas,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;
    use std::cell::RefCell;

    thread_local! {
        static ENTITY_COUNTER: RefCell<u32> = RefCell::new(0);
    }

    fn create_entity() -> Entity {
        ENTITY_COUNTER.with(|counter| {
            let index = *counter.borrow();
            *counter.borrow_mut() += 1; // Increment the counter for the next entity
            Entity::from_raw_u32(index).unwrap()
        })
    }

    #[test]
    fn test_move_from_stock_to_area() {
        let mut population = Population::new(4);
        let token_1 = create_entity();
        let token_2 = create_entity();
        let mut token_stock = TokenStock::new(47, vec![token_1, token_2]);
        let mut player_areas = PlayerAreas::default();
        let player = create_entity();
        let area = create_entity();

        move_from_stock_to_area(
            player,
            area,
            1,
            &mut population,
            &mut token_stock,
            &mut player_areas,
        );

        assert!(population.has_player(&player));
        assert_eq!(token_stock.tokens_in_stock(), 1);
    }

    #[test]
    fn test_return_all_tokens_from_area_to_player() {
        let mut population = Population::new(4);
        let token_1 = create_entity();
        let token_2 = create_entity();
        let mut token_stock = TokenStock::new(47, vec![token_1, token_2]);
        let mut player_areas = PlayerAreas::default();
        let player = create_entity();
        let area = create_entity();

        move_from_stock_to_area(
            player,
            area,
            2,
            &mut population,
            &mut token_stock,
            &mut player_areas,
        );
        return_all_tokens_from_area_to_player(
            &player,
            &area,
            &mut population,
            &mut token_stock,
            &mut player_areas,
        );

        assert!(token_stock.tokens_in_stock() >= 2);
        assert!(!player_areas.contains(area));
    }

    #[test]
    fn test_replace_city_with_tokens_for_conflict_city_removed() {
        let area_entity = create_entity();
        let mut population = Population::new(4);
        let city_token = create_entity();
        let mut city_stock = CityTokenStock::new(7, vec![]);
        let mut token_stock = TokenStock::new(47, vec![]);
        let mut player_cities = PlayerCities::default();
        let mut player_areas = PlayerAreas::default();
        let built_city = BuiltCity {
            player: create_entity(),
            city: city_token,
        };

        // Build city first to simulate a scenario
        player_cities.build_city_in_area(area_entity, city_token);

        replace_city_with_tokens_for_conflict(
            area_entity,
            &mut population,
            &built_city,
            &mut city_stock,
            &mut token_stock,
            &mut player_cities,
            &mut player_areas,
        );

        // Check that the city has been removed from the player cities
        assert!(!player_cities.has_city_in(area_entity));
        // Check that the city token has been returned to the stock
        assert_eq!(city_stock.get_token_from_stock().unwrap(), city_token);
    }
}
