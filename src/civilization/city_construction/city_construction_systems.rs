use crate::civilization::city_construction::city_construction_components::IsBuilding;
use crate::civilization::city_construction::city_construction_events::{BuildCityCommand, EndPlayerCityConstruction};
use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::general::general_components::population::Population;
use crate::civilization::general::general_components::*;
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::civilization::map::map_plugin::AvailableFactions;
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{debug, default, Commands, Entity, EventReader, EventWriter, NextState, Query, Res, ResMut, SpriteBundle, Transform, Vec3, With};

pub fn city_building_gate(
    query: Query<&IsBuilding>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if query.is_empty() {
        debug!("No one is building cities, moving on!");
        next_state.set(GameActivity::RemoveSurplusPopulation);
    }
}

pub fn build_city(
    mut command: EventReader<BuildCityCommand>,
    mut city_population: Query<(&mut Population, &Transform)>,
    mut return_tokens: EventWriter<ReturnTokenToStock>,
    mut player_query: Query<(&mut CityTokenStock, &mut PlayerAreas, &mut PlayerCities, &Faction)>,
    mut commands: Commands,
    mut recalculate_player_moves: EventWriter<RecalculatePlayerMoves>,
    game_factions: Res<AvailableFactions>,
) {
    for build_city in command.read() {
        if let Ok((mut city_stock, mut player_areas, mut player_cities, faction)) = player_query.get_mut(build_city.player) {
            if let Ok((mut population, area_transform)) = city_population.get_mut(build_city.area) {
                // we shall return all tokens
                for token in population.remove_all_tokens() {
                    return_tokens.send(ReturnTokenToStock {
                        token_entity: token,
                    });
                }

                if let Some(city_token) = city_stock.get_token_from_stock() {
                    player_areas.remove_area(build_city.area);
                    player_cities.build_city_in_area(build_city.area, city_token);
                    commands.entity(build_city.area)
                        .insert(BuiltCity {
                            player: build_city.player,
                            city: city_token,
                        });
                    commands.entity(city_token)
                        .insert(SpriteBundle {
                            texture: game_factions.faction_city_icons.get(&faction.faction).unwrap().clone(),
                            transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                                .with_translation(area_transform.translation),
                            ..default()
                        });
                    recalculate_player_moves.send(RecalculatePlayerMoves::new(build_city.player));
                }
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
        commands.entity(event.player)
            .remove::<AvailableMoves>();
    }
}
