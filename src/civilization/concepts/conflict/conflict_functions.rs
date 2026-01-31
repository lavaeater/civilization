use crate::civilization::components::population::Population;
use crate::civilization::components::ReturnTokenToStock;
use crate::civilization::functions::{remove_n_tokens_from_each_player, return_all_but_n_tokens_to_stock_for_player, return_all_tokens_from_area_for_player, return_all_tokens_to_stock};
use bevy::prelude::{Commands, Entity};

pub fn handle_all_lengths_equal(
    players: &Vec<Entity>,
    population: &mut Population,
    commands: &mut Commands,
) {
    let mut token_rounds = 1;
    let must_remove = population.total_population() - population.max_population;
    while token_rounds * population.number_of_players() < must_remove {
        token_rounds += 1;
    }

    //debug!("Removing {} tokens from each player", token_rounds);
    remove_n_tokens_from_each_player(players, population, commands, token_rounds);
}

pub fn handle_unequal_lengths(
    players: &mut Vec<Entity>,
    population: &mut Population,
    commands: &mut Commands,
) {
    // Sort players by their token count (from most to least)
    players.sort_by(|a, b| {
        population
            .population_for_player(*b)
            .cmp(&population.population_for_player(*a))
    });

    // Continue removing tokens while the total population is greater than max_population
    // and more than one player still has tokens
    while population.total_population() > population.max_population && players.len() > 1 {
        //debug!("Total Population: {}, Max Population: {}", population.total_population(), population.max_population);
        let current_player = players.pop().unwrap();
        //debug!("current player has {} tokens", population.population_for_player(current_player));

        // Remove 1 token from the current player
        for token in population
            .remove_tokens_from_area(&current_player, 1)
            .unwrap_or_default()
        {
            commands.entity(token).insert(ReturnTokenToStock);
        }

        // Check if the current player still has tokens, if so, put them back in the queue
        if population.population_for_player(current_player) > 0 {
            //debug!("Player still has tokens {}, putting them back in the queue", population.population_for_player(current_player));
            players.insert(0, current_player); // Put back in the queue if they still have tokens
        }

        // If only one player remains with tokens, stop the process
        if players.len() == 1 && population.population_for_player(players[0]) > 0 {
            //debug!("Only one player remains with tokens, stopping the process");
            break;
        }

        // Stop if the total population is now less than or equal to the max_population
        if population.total_population() <= population.max_population {
            //debug!("Total population is now less than or equal to the max population, stopping the process");
            break;
        }
    }
}

pub fn handle_max_pop_is_one_conflicts(
    players: &mut Vec<Entity>,
    population: &mut Population,
    commands: &mut Commands,
) {
    // Sort players by their population size (from highest to lowest)
    players.sort_by(|a, b| {
        population
            .population_for_player(*b)
            .cmp(&population.population_for_player(*a))
    });
    //debug!("Max pop one conflict!");
    // If all players have the same number of tokens
    if population.all_lengths_equal() {
        //debug!("All players have the same number of tokens - we remove all tokens!");
        // Remove all tokens from every player
        return_all_tokens_to_stock(population, commands);
    } else {
        //debug!("All players do not have the same number of tokens");
        // Find the player with the highest population
        let largest_player = players[0];

        // Remove all but 2 tokens from the player with the largest population
        return_all_but_n_tokens_to_stock_for_player(population, &largest_player, 2, commands);

        // Remove all tokens from all other players
        for player in players.iter().skip(1) {
            // Skip the largest player
            //debug!("Removing all tokens from other players");
            return_all_tokens_from_area_for_player(population, player, commands);
        }
    }
}
