use crate::civilization::game_phases::game_activity::GameActivity;
use bevy::app::App;
use bevy::prelude::{Entity, EventWriter, NextState, OnEnter, Plugin, Query, ResMut};
use crate::civilization::general::components::{Area, Population, Stock};
use crate::civilization::general::events::ReturnTokenToStock;

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), resolve_conflicts)
        ;
    }
}

fn resolve_conflicts(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut pop_query: Query<(&mut Population, &Area)>,
    mut return_token: EventWriter<ReturnTokenToStock>
) {
    let mut conflict_zones = pop_query.iter_mut().filter(|(pop, area)| {
        pop.player_tokens.keys().len() > 1 && pop.total_population > area.max_population
    }).collect::<Vec<_>>();

    for (mut pop, area) in conflict_zones {
        // let mut players = pop.player_tokens.keys().collect::<Vec<_>>();
        //
        // players.sort_by(|a, b| pop.player_tokens[*b].len().cmp(&pop.player_tokens[*a].len()));
        /*
        The rules of the game are simple. There is an algorithm to the number of tokens to be left after removing enough

        If player B has ONE token and player A has more than one, player B loses the token and conflict is resolved
         */

        if area.max_population % 2 == 0 && pop.player_tokens.len() == 2 {
            let mut player_tokens = pop.player_tokens.clone();
            let mut keys = pop.player_tokens.keys().cloned().collect::<Vec<Entity>>();
            let key_a = &keys.remove(0);
            let key_b = &keys.remove(0);

            let same_length = player_tokens.get(key_a).unwrap().len() == player_tokens.get(key_b).unwrap().len();

            if same_length {
                while pop.total_population > area.max_population {
                    pop.total_population -= 2;
                    if let Some(vec_a) = pop.player_tokens.get_mut(key_a) {
                        if let Some(token_entity) = vec_a.pop() {
                            return_token.send(ReturnTokenToStock { token_entity });
                        }
                    }

                    if let Some(vec_b) = pop.player_tokens.get_mut(key_b) {
                        if let Some(token_entity) = vec_b.pop() {
                            return_token.send(ReturnTokenToStock { token_entity });
                        }
                    }
                }
            } else {
                // Determine order
                let mut temp_map = map.into_iter().collect();

                // Step 2: Sort the Vec by the usize value in descending order
                vec.sort_by(|a, b| b.1.cmp(&a.1));

                // Step 3: Extract the Entity values from the sorted Vec
                let sorted_entities: Vec<Entity> = vec.into_iter().map(|(entity, _)| entity).collect();




                while pop.total_population > area.max_population {
                    pop.total_population -= 1;
                    if let Some(vec_a) = pop.player_tokens.get_mut(key_a) {
                        if let Some(token_entity) = vec_a.pop() {
                            return_token.send(ReturnTokenToStock { token_entity });
                        }
                    }

                    if let Some(vec_b) = pop.player_tokens.get_mut(key_b) {
                        if let Some(token_entity) = vec_b.pop() {
                            return_token.send(ReturnTokenToStock { token_entity });
                        }
                    }
                }
            }
        }
    }
    next_state.set(GameActivity::PopulationExpansion);
}