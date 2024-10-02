use bevy::prelude::{debug, Commands, Entity, EventWriter, Has, NextState, Query, ResMut, With};
use bevy::core::Name;
use bevy_console::PrintConsoleLine;
use crate::civilization::conflict::conflict_components::UnresolvedConflict;
use crate::civilization::general::general_components::{BuiltCity, Population};
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;

pub fn resolve_conflicts(
    mut conflict_zones: Query<(Entity, &Name, &mut Population, Has<BuiltCity>), With<UnresolvedConflict>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands) {
    for (area_entity, _name, mut population, has_city) in conflict_zones.iter_mut() {
        let temp_map = population.player_tokens.clone();
        let mut players = temp_map.keys().copied().collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        if population.max_population == 1 {
            if population.number_of_players() == 2 {
                handle_city_conflict(&mut players, &mut population, &mut return_token);
            } else {
                debug!("WEIRD CORNER CASE AAAH!");
            }
        } else if population.all_lengths_equal() {
            handle_all_lengths_equal(&players, &mut population, &mut return_token);
        } else {
            handle_unequal_lengths(&mut players, &mut population, &mut return_token);
        }

        commands.entity(area_entity).remove::<UnresolvedConflict>();
    }
}

fn handle_all_lengths_equal(
    players: &Vec<Entity>,
    population: &mut Population,
    return_token: &mut EventWriter<ReturnTokenToStock>
) {
    let mut token_rounds = 1;
    let must_remove = population.total_population() - population.max_population;
    while token_rounds * population.number_of_players() < must_remove {
        token_rounds += 1;
    }

    for player in players {
        for token in population.remove_tokens_from_area(*player, token_rounds).unwrap_or_default() {
            return_token.send(ReturnTokenToStock::new(token));
        }
    }
}

fn handle_unequal_lengths(
    players: &mut Vec<Entity>,
    population: &mut Population,
    return_token: &mut EventWriter<ReturnTokenToStock>
) {
    while population.total_population() > population.max_population {
        let current_player = players.pop().unwrap();
        for token in population.remove_tokens_from_area(current_player, 1).unwrap_or_default() {
            return_token.send(ReturnTokenToStock::new(token));
        }
        if population.has_player(current_player) {
            players.insert(0, current_player); // Put the player back in the queue if they still have tokens
        }
    }
}

fn handle_city_conflict(
    players: &mut Vec<Entity>,
    population: &mut Population,
    return_token: &mut EventWriter<ReturnTokenToStock>
) {
    let player_one = players.pop().unwrap();
    let player_two = players.pop().unwrap();

    if population.all_lengths_equal() {
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
}


pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population, Has<BuiltCity>)>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    pop_query.iter().filter(|(_, _, pop, has_city)| {
        pop.is_conflict_zone(*has_city)
    }).for_each(|(conflict_zone, name, _, _)| {
        write_line.send(PrintConsoleLine::new(format!("Conflict zone found: {:?}", name)));
        commands.entity(conflict_zone).insert(UnresolvedConflict);
    });
    next_state.set(GameActivity::CityConstruction);
}
