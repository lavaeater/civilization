use bevy::prelude::{Commands, Entity, EventReader, EventWriter, Has, NextState, Query, ResMut, With, Without};
use crate::civilization::cities::components::{CityBuildTargets, DoneBuilding};
use crate::civilization::cities::events::{BuildCity, EndCityConstructionActivity};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::{BuiltCity, CitySite, CityTokenStock, Population};
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
) {
    for build_city in command.iter() {
        println!("Build city for player {:?} in area {:?}", build_city.player, build_city.area);
    }
}

pub fn setup_players_and_cities(
    player_query: Query<(Entity, &CityTokenStock), With<Player>>,
    query: Query<(Entity, &Population, &CitySite), Without<BuiltCity>>,
    mut commands: Commands,
) {
    for (player_entity, city_token_stock) in player_query.iter() {
        if city_token_stock.tokens.is_empty() {
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
        next_state.set(GameActivity::PopulationExpansion);
    }
}