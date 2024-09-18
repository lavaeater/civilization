use crate::civilization::city_construction::city_construction_components::IsBuilding;
use crate::civilization::city_construction::city_construction_events::{BuildCityCommand, EndPlayerCityConstruction};
use crate::civilization::general::general_components::{BuiltCity, CityTokenStock, PlayerAreas, PlayerCities, Population};
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;

pub fn city_building_gate(
    query: Query<&IsBuilding>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if query.is_empty() {
        next_state.set(GameActivity::RemoveSurplusPopulation);
    }
}

pub fn build_city(
    mut command: EventReader<BuildCityCommand>,
    mut city_token_stock: Query<&mut CityTokenStock>,
    mut city_population: Query<&mut Population>,
    mut return_tokens: EventWriter<ReturnTokenToStock>,
    mut player_cities_and_areas: Query<(&mut PlayerAreas, &mut PlayerCities)>,
    mut commands: Commands,
    mut recalculate_player_moves: EventWriter<RecalculatePlayerMoves>
) {
    for build_city in command.read() {
        if let Ok(mut city_stock) = city_token_stock.get_mut(build_city.player) {
            if let Ok(mut population) = city_population.get_mut(build_city.area) {
                // we shall return all tokens
                let tokens_to_return = population.player_tokens.values().flatten().map(|t| t.clone()).collect::<Vec<Entity>>();
                population.player_tokens.clear();
                for token in tokens_to_return {
                    return_tokens.send(ReturnTokenToStock {
                        token_entity: token,
                    });
                }
            }
            if let Some(city_token) = city_stock.get_token_from_stock() {
                if let Ok((mut player_areas, mut player_cities)) = player_cities_and_areas.get_mut(build_city.player) {
                    player_areas.remove_area(build_city.area);
                    player_cities.build_city_in_area(build_city.area, city_token);
                }
                commands.entity(build_city.area)
                    .insert(BuiltCity {
                        player: build_city.player,
                        city: city_token,
                    });
                recalculate_player_moves.send(RecalculatePlayerMoves::new(build_city.player));
            }
        }
    }
}

pub fn on_enter_city_construction(
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for player_entity in player_query.iter() {
        commands.entity(player_entity)
            .insert(IsBuilding);
    }
}

pub fn end_player_city_construction(
    mut end_activity: EventReader<EndPlayerCityConstruction>,
    mut commands: Commands,
) {
    for event in end_activity.read() {
        commands.entity(event.player)
            .remove::<IsBuilding>();
    }
}
