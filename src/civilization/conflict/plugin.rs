use std::cmp::Ordering;
use crate::civilization::game_phases::game_activity::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, Commands, Component, Entity, EventWriter, IntoSystemConfigs, NextState, OnEnter, Plugin, Query, Reflect, ResMut, Update, Without};
use crate::civilization::general::components::{Area, Population, Stock};
use crate::civilization::general::events::ReturnTokenToStock;

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), find_conflict_zones)
            .add_systems(
                Update, (
                    resolve_conflicts.run_if(in_state(GameActivity::Conflict))
                ),
            )
        ;
    }
}

#[derive(Component, Debug, Reflect)]
pub struct UnresolvedConflict;

fn resolve_conflicts() {}

fn find_conflict_zones(
    pop_query: Query<(Entity, &Population, &Area)>,
    mut commands: Commands,
) {
    pop_query.iter().filter(|(conflict_zone, pop, area)| {
        pop.player_tokens.keys().len() > 1 && pop.total_population > area.max_population
    }).for_each(|(conflict_zone, _, _)| {
        commands.entity(conflict_zone).insert(UnresolvedConflict);
    });

    // for (mut pop, area) in conflict_zones {
    //     // let mut players = pop.player_tokens.keys().collect::<Vec<_>>();
    //     //
    //     // players.sort_by(|a, b| pop.player_tokens[*b].len().cmp(&pop.player_tokens[*a].len()));
    //     /*
    //     The rules of the game are simple. There is an algorithm to the number of tokens to be left after removing enough
    //
    //     If player B has ONE token and player A has more than one, player B loses the token and conflict is resolved
    //      */
    //
    //     if area.max_population % 2 == 0 && pop.player_tokens.len() == 2 {
    //         let mut player_tokens = pop.player_tokens.clone();
    //         let mut keys = pop.player_tokens.keys().cloned().collect::<Vec<Entity>>();
    //         let key_a = &keys.remove(0);
    //         let key_b = &keys.remove(0);
    //
    //         let player_tokens_a = player_tokens.get(key_a).unwrap();
    //         let player_tokens_b = player_tokens.get(key_b).unwrap();
    //
    //         let compared = player_tokens_a.len().cmp(&player_tokens_b.len());
    //
    //         match compared {
    //             Ordering::Less => {
    //                 // player_a has less tokens
    //
    //             }
    //             Ordering::Equal => {
    //                 while pop.total_population > area.max_population {
    //                     pop.total_population -= 2;
    //                     if let Some(vec_a) = pop.player_tokens.get_mut(key_a) {
    //                         if let Some(token_entity) = vec_a.pop() {
    //                             return_token.send(ReturnTokenToStock { token_entity });
    //                         }
    //                     }
    //
    //                     if let Some(vec_b) = pop.player_tokens.get_mut(key_b) {
    //                         if let Some(token_entity) = vec_b.pop() {
    //                             return_token.send(ReturnTokenToStock { token_entity });
    //                         }
    //                     }
    //                 }
    //             }
    //             Ordering::Greater => {
    //                 // player_b has less tokens
    //             }
    //         }
    //
    //         //     } else {
    //         //         // Determine order
    //         //         let mut temp_map = map.into_iter().collect();
    //         //
    //         //         // Step 2: Sort the Vec by the usize value in descending order
    //         //         vec.sort_by(|a, b| b.1.cmp(&a.1));
    //         //
    //         //         // Step 3: Extract the Entity values from the sorted Vec
    //         //         let sorted_entities: Vec<Entity> = vec.into_iter().map(|(entity, _)| entity).collect();
    //         //
    //         //
    //         //
    //         //
    //         //         while pop.total_population > area.max_population {
    //         //             pop.total_population -= 1;
    //         //             if let Some(vec_a) = pop.player_tokens.get_mut(key_a) {
    //         //                 if let Some(token_entity) = vec_a.pop() {
    //         //                     return_token.send(ReturnTokenToStock { token_entity });
    //         //                 }
    //         //             }
    //         //
    //         //             if let Some(vec_b) = pop.player_tokens.get_mut(key_b) {
    //         //                 if let Some(token_entity) = vec_b.pop() {
    //         //                     return_token.send(ReturnTokenToStock { token_entity });
    //         //                 }
    //         //             }
    //         //         }
    //         //     }
    //         // }
    //     }
    // }
    // next_state.set(GameActivity::PopulationExpansion);
}