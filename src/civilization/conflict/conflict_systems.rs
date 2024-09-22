use bevy::prelude::{Commands, Entity, EventWriter, NextState, Query, ResMut, With};
use bevy::core::Name;
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::conflict::conflict_components::UnresolvedConflict;
use crate::civilization::general::general_components::Population;
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;

pub fn resolve_conflicts(
    mut conflict_zones: Query<(Entity, &Name, &mut Population), With<UnresolvedConflict>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands) {
    for (area_entity, _name, mut population) in conflict_zones.iter_mut() {
        let temp_map = population.player_tokens.clone();
        let mut players = temp_map.keys().copied().collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        if population.max_population == 1 {
            if population.number_of_players() == 2 {
                let player_one = players.pop().unwrap();
                let player_two = players.pop().unwrap();

                if population.all_lengths_equal() {
                    //THey should have ZERO pop left!
                    for token in population.remove_all_but_n_tokens(player_one, 0).unwrap_or_default() {
                        return_token.send(ReturnTokenToStock::new(token));
                    }
                    for token in population.remove_all_but_n_tokens(player_two, 0).unwrap_or_default() {
                        return_token.send(ReturnTokenToStock::new(token));
                    }
                } else if population.population_for_player(player_one) > population.population_for_player(player_two) {
                    for token in population.remove_all_but_n_tokens(player_one, 2).unwrap_or_default() {
                        return_token.send(ReturnTokenToStock::new(token));
                    }
                    for token in population.remove_all_but_n_tokens(player_two, 0).unwrap_or_default() {
                        return_token.send(ReturnTokenToStock::new(token));
                    }
                } else {
                    for token in population.remove_all_but_n_tokens(player_one, 0).unwrap_or_default() {
                        return_token.send(ReturnTokenToStock::new(token));
                    }
                    for token in population.remove_all_but_n_tokens(player_two, 2).unwrap_or_default() {
                        return_token.send(ReturnTokenToStock::new(token));
                    }
                }
            } else {
                println!("This is a weird corner case. We have a conflict zone with more than 2 players");
                //Weird fucking corner case where we have an odd number of players... aaah
            }
        } else {
            match population.max_population % 2 {
                1 => {
                    // the max pop is odd. This means something.
                    if population.all_lengths_equal() {
                        /*
                        I get it. 
                        everyone has to remove one token per player until we 
                        are below max_pop
                        
                        So that means that we do
                         */
                        let mut token_rounds = 1;
                        let must_remove = population.total_population() - population.max_population;
                        while token_rounds * population.number_of_players() < must_remove {
                            token_rounds += 1;
                        }

                        let must_remove = token_rounds;

                        for player in players {
                            for token in population.remove_tokens_from_area(player, must_remove).unwrap_or_default() {
                                return_token.send(ReturnTokenToStock::new(token));
                            }
                        }
                    } else {
                        while population.total_population() > population.max_population {
                            let current_player = players.pop().unwrap();
                            for token in population.remove_tokens_from_area(current_player, 1).unwrap_or_default() {
                                return_token.send(ReturnTokenToStock::new(token));
                            }
                            if population.has_player(current_player) {
                                players.insert(0, current_player);
                            }
                        }
                    }
                }
                _ => {
                    if population.all_lengths_equal() {
                        /*
                        I get it. 
                        everyone has to remove one token per player until we 
                        are below max_pop
                        
                        So that means that we do
                         */
                        let mut token_rounds = 1;
                        let must_remove = population.total_population() - population.max_population;
                        while token_rounds * population.number_of_players() < must_remove {
                            token_rounds += 1;
                        }

                        let must_remove = token_rounds;

                        for player in players {
                            if let Some(tokens) = population.remove_tokens_from_area(player, must_remove) {
                                for token in tokens {
                                    return_token.send(ReturnTokenToStock::new(token));
                                }
                            }
                        }
                    } else {
                        while population.total_population() > population.max_population {
                            let current_player = players.pop().unwrap();
                            for token in population.remove_tokens_from_area(current_player, 1).unwrap_or_default() {
                                return_token.send(ReturnTokenToStock::new(token));
                            }
                            if population.has_player(current_player) {
                                players.insert(0, current_player);
                            }
                        }
                    }
                }
            }
            commands.entity(area_entity).remove::<UnresolvedConflict>();
        }
    }
}

pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population)>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    pop_query.iter().filter(|(_, _, pop)| {
        pop.is_conflict_zone()
    }).for_each(|(conflict_zone, name, _)| {
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Conflict zone found: {:?}", name))));
        commands.entity(conflict_zone).insert(UnresolvedConflict);
    });
    next_state.set(GameActivity::CityConstruction);
}
