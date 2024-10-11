use crate::civilization::conflict::conflict_components::{UnresolvedCityConflict, UnresolvedConflict};
use crate::civilization::general::general_components::{BuiltCity, Population};
use crate::civilization::general::general_events::ReturnTokenToStock;
use bevy::core::Name;
use bevy::prelude::{Commands, Entity, EventWriter, OnAdd, Query, Trigger};

pub fn on_add_unresolved_conflict(
    trigger: Trigger<OnAdd, UnresolvedConflict>,
    mut areas: Query<(Entity, &Name, &mut Population)>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands,
) {
    if let Ok((area_entity, _name, mut population)) = areas.get_mut(trigger.entity()) {
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
    players.sort_by(|a, b| population.population_for_player(*b).cmp(&population.population_for_player(*a)));

    // Continue removing tokens while the total population is greater than max_population
    // and more than one player still has tokens
    while population.total_population() > population.max_population && players.len() > 1 {
        let current_player = players.pop().unwrap();

        // Remove 1 token from the current player
        for token in population.remove_tokens_from_area(current_player, 1).unwrap_or_default() {
            return_token.send(ReturnTokenToStock::new(token));
        }

        // Check if the current player still has tokens, if so, put them back in the queue
        if population.population_for_player(current_player) > 0 {
            players.insert(0, current_player); // Put back in the queue if they still have tokens
        }

        // If only one player remains with tokens, stop the process
        if players.len() == 1 && population.population_for_player(players[0]) > 0 {
            break;
        }

        // Stop if the total population is now less than or equal to the max_population
        if population.total_population() <= population.max_population {
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

pub fn on_add_unresolved_city_conflict(
    trigger: Trigger<OnAdd, UnresolvedCityConflict>,
    mut areas: Query<(Entity, &Name, &mut Population, &BuiltCity)>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands) {
    if let Ok((area_entity,
                  _name,
                  mut population,
                  built_city)) = areas.get_mut(trigger.entity()) {
        let mut other_players = population.players();
        other_players.remove(&built_city.player);
        for player in other_players {
            if population.population_for_player(player) > 6 {
                
            }
        }
        /*
        1. Does the non-city players have 7 or more tokens in this area?
            ## No: 
                1. Eliminate all these tokens, return them to the player's stock
            ## Yes: 
                1. Eliminate the city, return to stock
                2. Get six (or fewer if player does not have six tokens in stock)
                3. Mark as a completely regular conflict zone.
         2. Profit!
         */
        commands.entity(area_entity).remove::<UnresolvedCityConflict>();
    }
}