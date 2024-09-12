use bevy::prelude::{Commands, Entity, EventReader, EventWriter, Has, NextState, Query, ResMut, With, Without};
use crate::civilization::city_construction::city_construction_components::{CityBuildTargets, DoneBuilding};
use crate::civilization::city_construction::city_construction_events::{BuildCity, EndCityConstructionActivity};
use crate::civilization::general::general_components::{BuiltCity, CitySite, CityTokenStock, PlayerAreas, PlayerCities, Population};
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;
use crate::player::Player;

pub fn check_if_done_building(
    query: Query<(Entity, Has<DoneBuilding>), With<Player>>,
    mut end_activity: EventWriter<EndCityConstructionActivity>
) {
    let all_players_count = query.iter().len();
    let done_player_count = query.iter().filter(|(_, done_building)| *done_building).count();

    if all_players_count == done_player_count {
        end_activity.send(EndCityConstructionActivity {});
    }
}

pub fn build_city(
    mut command: EventReader<BuildCity>,
    mut city_token_stock: Query<&mut CityTokenStock>,
    mut city_population: Query<&mut Population>,
    mut return_tokens: EventWriter<ReturnTokenToStock>,
    mut player_cities_and_areas: Query<(&mut PlayerAreas, &mut PlayerCities)>,
    mut commands: Commands,
) {
    for build_city in command.read() {
        if let Ok(mut city_stock) = city_token_stock.get_mut(build_city.player) {
            if let Ok(mut population) = city_population.get_mut(build_city.area) {
                // we shall return all tokens
                let tokens_to_return = population.player_tokens.values().flatten().map(|t|t.clone()).collect::<Vec<Entity>>();
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

                println!("Build city for player {:?} in area {:?}", build_city.player, build_city.area);
                commands.entity(build_city.area)
                    .insert(BuiltCity {
                        player: build_city.player,
                        city: city_token,
                    });
            }
        }
    }
}

pub fn setup_players_and_cities(
    player_query: Query<(Entity, &CityTokenStock), With<Player>>,
    query: Query<(Entity, &Population, &CitySite), Without<BuiltCity>>,
    mut commands: Commands,
) {
    for (player_entity, city_token_stock) in player_query.iter() {
        if city_token_stock.is_empty() {
            commands.entity(player_entity)
                .insert(DoneBuilding {});
            continue;
        } else {
            let targets: Vec<Entity> = query.iter().filter(|(_, population, _)| {
                return if population.player_tokens.contains_key(&player_entity) {
                    let tokens = population.player_tokens.get(&player_entity).unwrap();
                    tokens.len() >= 6
                } else {
                    false
                };
            }).map(|(entity, _, _)| entity).collect();

            if targets.is_empty() {
                commands.entity(player_entity)
                    .insert(DoneBuilding {});
            } else {
                commands.entity(player_entity)
                    .insert(CityBuildTargets {
                        targets
                    });
            }
        }
    }
}

pub fn end_city_construction_activity(
    mut end_activity: EventReader<EndCityConstructionActivity>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in end_activity.read() {
        for player_entity in player_query.iter() {
            commands.entity(player_entity)
                .remove::<CityBuildTargets>();
            commands.entity(player_entity)
                .remove::<DoneBuilding>();
        }
        next_state.set(GameActivity::RemoveSurplusPopulation);
    }
}