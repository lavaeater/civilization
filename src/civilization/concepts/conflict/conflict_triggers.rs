use crate::civilization::components::*;
use crate::civilization::concepts::conflict::conflict_components::*;
use crate::civilization::concepts::conflict::conflict_functions::*;
use crate::civilization::functions::{replace_city_with_tokens_for_conflict, return_all_tokens_from_area_to_player};
use bevy::prelude::{Add, Commands, Entity, Name, NextState, On, Query, ResMut};
use std::cmp::Ordering;
use crate::civilization::ConflictCounterResource;
use crate::GameActivity;

pub fn on_add_unresolved_conflict(
    trigger: On<Add, UnresolvedConflict>,
    mut areas: Query<(Entity, &Name, &mut Population)>,
    mut commands: Commands,
    mut conflict_counter_resource: ResMut<ConflictCounterResource>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if let Ok((area_entity, _name, mut population)) = areas.get_mut(trigger.event().entity) {
        let temp_map = population.player_tokens().clone();
        let mut players = temp_map.keys().copied().collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        if population.max_population == 1 {
            handle_max_pop_is_one_conflicts(&mut players, &mut population, &mut commands);
        } else if population.all_lengths_equal() {
            handle_all_lengths_equal(&players, &mut population, &mut commands);
        } else {
            handle_unequal_lengths(&mut players, &mut population, &mut commands);
        }

        commands.entity(area_entity).remove::<UnresolvedConflict>();
        conflict_counter_resource.0 -= 1;
        if conflict_counter_resource.0 == 0 {
            next_state.set(GameActivity::CityConstruction);
        }
    }
}

pub fn on_add_unresolved_city_conflict(
    trigger: On<Add, UnresolvedCityConflict>,
    mut areas: Query<(Entity, &Name, &mut Population, &BuiltCity)>,
    mut player_with_city: Query<(
        &mut CityTokenStock,
        &mut TokenStock,
        &mut PlayerCities,
        &mut PlayerAreas,
    )>,
    mut commands: Commands,
    mut conflict_counter_resource: ResMut<ConflictCounterResource>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if let Ok((area_entity, _name, mut population, built_city)) = areas.get_mut(trigger.event().entity) {
        let mut other_players = population.players();
        other_players.remove(&built_city.player);
        if other_players
            .iter()
            .any(|p| population.population_for_player(*p) > 6)
        {
            match other_players.len().cmp(&1) {
                Ordering::Less => {
                }
                Ordering::Equal => {
                    if let Ok((
                        mut city_stock,
                        mut token_stock,
                        mut player_cities,
                        mut player_areas,
                    )) = player_with_city.get_mut(built_city.player)
                    {
                        commands.entity(area_entity).remove::<BuiltCity>();
                        replace_city_with_tokens_for_conflict(
                            area_entity,
                            &mut population,
                            built_city,
                            &mut city_stock,
                            &mut token_stock,
                            &mut player_cities,
                            &mut player_areas,
                        );
                        commands.entity(area_entity).insert(UnresolvedConflict);
                    }
                }
                Ordering::Greater => {
                    /*
                    This is a super special case that requires handling - battles between other parties are to be resolved first, which
                    we incidentally actually CAN handle... yay!
                     */
                    commands.entity(trigger.event().entity).insert(UnresolvedConflict);
                }
            }
        } else {
            for player in other_players {
                if let Ok((_, mut token_stock, _, mut player_areas)) =
                    player_with_city.get_mut(player)
                {
                    return_all_tokens_from_area_to_player(
                        &player,
                        &area_entity,
                        &mut population,
                        &mut token_stock,
                        &mut player_areas,
                    );
                }
            }
        }
        commands
            .entity(area_entity)
            .remove::<UnresolvedCityConflict>();
        conflict_counter_resource.0 -= 1;
        if conflict_counter_resource.0 == 0 {
            next_state.set(GameActivity::CityConstruction);
        }
    }
}
