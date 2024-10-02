use crate::civilization::conflict::conflict_components::UnresolvedConflict;
use crate::civilization::general::general_components::{BuiltCity, Population};
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;
use bevy::core::Name;
use bevy::prelude::{debug, Commands, Entity, EventWriter, Has, NextState, Query, ResMut, With};
use bevy_console::PrintConsoleLine;

pub fn resolve_conflicts(
    mut conflict_zones: Query<(Entity, &Name, &mut Population, Has<BuiltCity>), With<UnresolvedConflict>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands) {
    for (area_entity, _name, mut population, _has_city) in conflict_zones.iter_mut() {
        let temp_map = population.player_tokens.clone();
        let mut players = temp_map.keys().copied().collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        if population.max_population == 1 {
            handle_max_pop_is_one_conflicts(&mut players, &mut population, &mut return_token);
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
    return_token: &mut EventWriter<ReturnTokenToStock>,
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
    return_token: &mut EventWriter<ReturnTokenToStock>,
) {
    // Sort players by their token count (from most to least)
    players.sort_by_key(|b| std::cmp::Reverse(population.population_for_player(*b)));

    // We will continue to remove tokens until only the player with the most tokens has non-zero tokens
    while players.len() > 1 {
        let current_player = players.pop().unwrap();

        // Remove 1 token from the current player
        for token in population.remove_tokens_from_area(current_player, 1).unwrap_or_default() {
            return_token.send(ReturnTokenToStock::new(token));
        }

        // Check if the current player still has tokens
        if population.population_for_player(current_player) > 0 {
            players.insert(0, current_player); // Put back in the queue if they still have tokens
        }

        // If only one player remains in the list, stop the process
        if players.len() == 1 && population.population_for_player(players[0]) > 0 {
            break;
        }
    }
}

fn handle_max_pop_is_one_conflicts(
    players: &mut Vec<Entity>,
    population: &mut Population,
    return_token: &mut EventWriter<ReturnTokenToStock>,
) {
    // Sort players by their population size (from highest to lowest)
    players.sort_by(|a, b| population.population_for_player(*b).cmp(&population.population_for_player(*a)));

    // If all players have the same number of tokens
    if population.all_lengths_equal() {
        // Remove all tokens from every player
        for player in players.iter() {
            for token in population.remove_all_but_n_tokens(*player, 0).unwrap_or_default() {
                return_token.send(ReturnTokenToStock::new(token));
            }
        }
    } else {
        // Find the player with the highest population
        let largest_player = players[0];

        // Remove all but 2 tokens from the player with the largest population
        for token in population.remove_all_but_n_tokens(largest_player, 2).unwrap_or_default() {
            return_token.send(ReturnTokenToStock::new(token));
        }

        // Remove all tokens from all other players
        for player in players.iter().skip(1) { // Skip the largest player
            for token in population.remove_all_but_n_tokens(*player, 0).unwrap_or_default() {
                return_token.send(ReturnTokenToStock::new(token));
            }
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
