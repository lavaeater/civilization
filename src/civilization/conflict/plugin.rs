use crate::civilization::game_phases::game_activity::GameActivity;
use bevy::app::App;
use bevy::prelude::{Commands, Component, Entity, EventWriter, NextState, OnEnter, OnExit, Plugin, Query, Reflect, ResMut, With};
use crate::civilization::general::components::{Area, Population};
use crate::civilization::general::events::ReturnTokenToStock;

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), find_conflict_zones)
            .add_systems(
                OnExit(GameActivity::Conflict), resolve_conflicts)
        ;
    }
}

#[derive(Component, Debug, Reflect)]
pub struct UnresolvedConflict;

fn resolve_conflicts(
    mut conflict_zones: Query<(Entity, &mut Population, &Area), With<UnresolvedConflict>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands,
) {
    for (area_entity, mut population, area) in conflict_zones.iter_mut() {
        let temp_map = population.player_tokens.clone();
        let mut players = temp_map.keys().map(|k| *k).collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        let all_lengths_equal = players.iter().all(|player| {
            temp_map[player].len() == temp_map[&players[0]].len()
        });

        while population.total_population > area.max_population {
            if all_lengths_equal {
                population.total_population -= players.len();
                for player in players.iter() {
                    if let Some(vec) = population.player_tokens.get_mut(player) {
                        if let Some(token_entity) = vec.pop() {
                            return_token.send(ReturnTokenToStock { token_entity });
                        }
                    }
                }
            } else {
                population.total_population -= 1;
                // pop the last player (has least tokens on this area)
                let player = players.pop().unwrap();
                if let Some(vec) = population.player_tokens.get_mut(&player) {
                    if let Some(token_entity) = vec.pop() {
                        return_token.send(ReturnTokenToStock { token_entity });
                    } else {
                        // has no more tokens here, remove altogether
                        population.player_tokens.remove(&player);
                    }
                }
                // insert the player back at the start of the line... yihaa.
                players.insert(0, player);
            }
        }
        commands.entity(area_entity).remove::<UnresolvedConflict>();
    }
}

fn find_conflict_zones(
    pop_query: Query<(Entity, &Population, &Area)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    pop_query.iter().filter(|(_, pop, area)| {
        pop.player_tokens.keys().len() > 1 && pop.total_population > area.max_population
    }).for_each(|(conflict_zone, _, _)| {
        commands.entity(conflict_zone).insert(UnresolvedConflict);
    });
    next_state.set(GameActivity::PopulationExpansion);
}
