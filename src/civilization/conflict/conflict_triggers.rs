use crate::civilization::city_support::city_support_events::EliminateCity;
use crate::civilization::conflict::conflict_components::{UnresolvedCityConflict, UnresolvedConflict};
use crate::civilization::general::general_components::population::Population;
use crate::civilization::general::general_components::*;
use crate::civilization::general::general_events::ReturnTokenToStock;
use bevy::core::Name;
use bevy::prelude::{debug, Commands, Entity, EventWriter, OnAdd, Query, Trigger};
use std::cmp::Ordering;

pub fn on_add_unresolved_conflict(
    trigger: Trigger<OnAdd, UnresolvedConflict>,
    mut areas: Query<(Entity, &Name, &mut Population)>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands,
) {
    debug!("On Add Triggered");
    if let Ok((area_entity, _name, mut population)) = areas.get_mut(trigger.entity()) {
        debug!("Lets resolve a regular conflict");
        let temp_map = population.player_tokens().clone();
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

    debug!("Removing {} tokens from each player", token_rounds);
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
        debug!("Total Population: {}, Max Population: {}", population.total_population(), population.max_population);
        let current_player = players.pop().unwrap();
        debug!("current player has {} tokens", population.population_for_player(current_player));

        // Remove 1 token from the current player
        for token in population.remove_tokens_from_area(current_player, 1).unwrap_or_default() {
            return_token.send(ReturnTokenToStock::new(token));
        }

        // Check if the current player still has tokens, if so, put them back in the queue
        if population.population_for_player(current_player) > 0 {
            debug!("Player still has tokens {}, putting them back in the queue", population.population_for_player(current_player));
            players.insert(0, current_player); // Put back in the queue if they still have tokens
        }

        // If only one player remains with tokens, stop the process
        if players.len() == 1 && population.population_for_player(players[0]) > 0 {
            debug!("Only one player remains with tokens, stopping the process");
            break;
        }

        // Stop if the total population is now less than or equal to the max_population
        if population.total_population() <= population.max_population {
            debug!("Total population is now less than or equal to the max population, stopping the process");
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
    debug!("Max pop one conflict!");
    // If all players have the same number of tokens
    if population.all_lengths_equal() {
        debug!("All players have the same number of tokens - we remove all tokens!");
        // Remove all tokens from every player
        for token in population.remove_all_tokens() {
            return_token.send(ReturnTokenToStock::new(token));
        }
    } else {
        debug!("All players do not have the same number of tokens");
        // Find the player with the highest population
        let largest_player = players[0];

        // Remove all but 2 tokens from the player with the largest population
        for token in population.remove_all_but_n_tokens(largest_player, 2).unwrap_or_default() {
            debug!("Removing token from largest player");
            return_token.send(ReturnTokenToStock::new(token));
        }

        // Remove all tokens from all other players
        for player in players.iter().skip(1) { // Skip the largest player
            debug!("Removing all tokens from other players");
            for token in population.remove_all_tokens_for_player(*player) {
                return_token.send(ReturnTokenToStock::new(token));
            }
        }
    }
}

pub fn on_add_unresolved_city_conflict(
    trigger: Trigger<OnAdd, UnresolvedCityConflict>,
    mut areas: Query<(Entity, &Name, &mut Population, &BuiltCity)>,
    mut player_with_city: Query<(&mut CityTokenStock, &mut TokenStock, &mut PlayerCities, &mut PlayerAreas)>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut eliminate_city: EventWriter<EliminateCity>,
    mut commands: Commands) {
    debug!("Lets resolve a City Conflict found");
    if let Ok((area_entity,
                  _name,
                  mut population,
                  built_city)) = areas.get_mut(trigger.entity()) {
        let mut other_players = population.players();
        other_players.remove(&built_city.player);
        if other_players.iter().any(|p| population.population_for_player(*p) > 6) {
            match other_players.len().cmp(&1) {
                Ordering::Less => {
                    debug!("There are no other players here, bro");
                }
                Ordering::Equal => {
                    debug!("There is one other player, we eliminate the city and resolve a regular conflict");
                    if let Ok((mut city_stock, mut token_stock, mut player_cities, mut player_areas)) = player_with_city.get_mut(built_city.player) {
                        commands.entity(area_entity).remove::<BuiltCity>();
                        player_cities.remove_city_from_area(area_entity);
                        city_stock.return_token_to_stock(built_city.city);

                        move_from_stock_to_area(built_city.player, area_entity, 6, &mut population, &mut token_stock, &mut player_areas);
                        
                        commands.entity(area_entity).insert(UnresolvedConflict);
                    }
                }
                Ordering::Greater => {
                    /*
                    This is a super special case that requires handling - battles between other parties are to be resolved first, which 
                    we incidentally actually CAN handle... yay!
                     */
                    debug!("There are more than one other player with six or more tokens!");
                    eliminate_city.send(EliminateCity::new(built_city.player, built_city.city, trigger.entity(), true));
                    commands.entity(trigger.entity()).insert(UnresolvedConflict);
                }
            }
        } else {
            debug!("There are no players with six or more tokens, we eliminate all tokens");
            // Kill them all
            for token in population.remove_all_tokens() {
                return_token.send(ReturnTokenToStock::new(token));
            }
        }
        commands.entity(area_entity).remove::<UnresolvedCityConflict>();
    }
}

fn move_from_stock_to_area(player: Entity, area: Entity, at_most_tokens: usize, population: &mut Population, token_stock: &mut TokenStock, player_areas: &mut PlayerAreas) {
    let tokens = token_stock.remove_at_most_n_tokens_from_stock(at_most_tokens).unwrap_or(vec![]);
    
    population.add_tokens_to_area(player, tokens.clone());
    player_areas.add_tokens_to_area(area, tokens);
}