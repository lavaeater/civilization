use crate::civilization::components::{Population, ReturnTokenToStock};
use crate::civilization::functions::{
    remove_n_tokens_from_each_player, return_all_but_n_tokens_to_stock_for_player,
    return_all_tokens_from_area_for_player, return_all_tokens_to_stock,
};
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
    players: &mut [Entity],
    population: &mut Population,
    commands: &mut Commands,
) {
    assert!(
        population.max_population == 1,
        "handle_max_pop_is_one_conflicts called with max_population != 1"
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::World;

    fn setup_world() -> World {
        World::default()
    }

    fn create_populated_area(
        world: &mut World,
        max_pop: usize,
        player_token_counts: &[(Entity, usize)],
    ) -> Population {
        let mut pop = Population::new(max_pop);
        for &(player, count) in player_token_counts {
            for _ in 0..count {
                let token = world.spawn_empty().id();
                pop.add_token_to_area(player, token);
            }
        }
        pop
    }

    // ========================================================================
    // handle_all_lengths_equal tests
    // ========================================================================

    #[test]
    fn test_all_equal_two_players_each_with_2_tokens_max_pop_2() {
        // Area max_pop=2, two players each with 2 tokens (total 4, need to remove 2)
        // Each player should lose 1 token
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 2, &[(p1, 2), (p2, 2)]);
        let players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_all_lengths_equal(&players, &mut pop, &mut commands);

        assert_eq!(pop.population_for_player(p1), 1);
        assert_eq!(pop.population_for_player(p2), 1);
        assert_eq!(pop.total_population(), 2);
    }

    #[test]
    fn test_all_equal_three_players_each_with_3_tokens_max_pop_3() {
        // Area max_pop=3, three players each with 3 tokens (total 9, need to remove 6)
        // token_rounds = 2 (2*3=6 >= 6), each player loses 2
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let p3 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 3, &[(p1, 3), (p2, 3), (p3, 3)]);
        let players = vec![p1, p2, p3];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_all_lengths_equal(&players, &mut pop, &mut commands);

        assert_eq!(pop.population_for_player(p1), 1);
        assert_eq!(pop.population_for_player(p2), 1);
        assert_eq!(pop.population_for_player(p3), 1);
        assert_eq!(pop.total_population(), 3);
    }

    #[test]
    fn test_all_equal_two_players_each_with_3_tokens_max_pop_4() {
        // Area max_pop=4, two players each with 3 tokens (total 6, need to remove 2)
        // token_rounds = 1 (1*2=2 >= 2), each player loses 1
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 4, &[(p1, 3), (p2, 3)]);
        let players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_all_lengths_equal(&players, &mut pop, &mut commands);

        assert_eq!(pop.population_for_player(p1), 2);
        assert_eq!(pop.population_for_player(p2), 2);
        assert_eq!(pop.total_population(), 4);
    }

    // ========================================================================
    // handle_unequal_lengths tests
    // ========================================================================

    #[test]
    fn test_unequal_two_players_removes_from_smallest_first() {
        // Area max_pop=2, p1 has 3 tokens, p2 has 1 token (total 4, need to remove 2)
        // Sorted most→least: [p1(3), p2(1)]
        // Pop from end: remove from p2 first (1→0), then p1 (3→2)
        // p2 eliminated, p1 has 2 tokens, only 1 player remains → stop
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 2, &[(p1, 3), (p2, 1)]);
        let mut players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_unequal_lengths(&mut players, &mut pop, &mut commands);

        // p2 should be eliminated (0 tokens), p1 should remain
        assert_eq!(pop.population_for_player(p2), 0);
        assert!(pop.population_for_player(p1) > 0);
        // Total should be at or below max
        assert!(pop.total_population() <= 2 || pop.number_of_players() <= 1);
    }

    #[test]
    fn test_unequal_three_players_terminates() {
        // Area max_pop=3, p1 has 4, p2 has 3, p3 has 2 (total 9, need to remove 6)
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let p3 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 3, &[(p1, 4), (p2, 3), (p3, 2)]);
        let mut players = vec![p1, p2, p3];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_unequal_lengths(&mut players, &mut pop, &mut commands);

        // Should terminate and reduce population
        assert!(
            pop.total_population() <= 3 || pop.number_of_players() <= 1,
            "Conflict should resolve: total_pop={}, num_players={}",
            pop.total_population(),
            pop.number_of_players()
        );
    }

    #[test]
    fn test_unequal_large_imbalance() {
        // Area max_pop=2, p1 has 10, p2 has 1 (total 11)
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 2, &[(p1, 10), (p2, 1)]);
        let mut players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_unequal_lengths(&mut players, &mut pop, &mut commands);

        // p2 eliminated, p1 remains as sole player
        assert_eq!(pop.population_for_player(p2), 0);
        assert!(pop.population_for_player(p1) > 0);
    }

    #[test]
    fn test_unequal_already_at_max_with_multiple_players() {
        // Edge case: total_population == max_population but 2 players
        // This shouldn't be called (is_conflict_zone checks has_too_many_tokens)
        // but let's verify it doesn't hang
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 4, &[(p1, 2), (p2, 2)]);
        let mut players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_unequal_lengths(&mut players, &mut pop, &mut commands);

        // Loop condition: total_pop(4) > max_pop(4) is false, so loop never executes
        assert_eq!(pop.total_population(), 4);
    }

    // ========================================================================
    // handle_max_pop_is_one_conflicts tests
    // ========================================================================

    #[test]
    fn test_max_pop_one_equal_tokens_all_removed() {
        // max_pop=1, two players each with 1 token → all removed
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 1, &[(p1, 1), (p2, 1)]);
        let mut players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_max_pop_is_one_conflicts(&mut players, &mut pop, &mut commands);

        assert_eq!(pop.total_population(), 0);
    }

    #[test]
    fn test_max_pop_one_unequal_largest_keeps_two_but_capped() {
        // max_pop=1, p1 has 3, p2 has 1
        // Largest (p1) keeps at most 2, all others removed
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 1, &[(p1, 3), (p2, 1)]);
        let mut players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_max_pop_is_one_conflicts(&mut players, &mut pop, &mut commands);

        // p1 keeps 2, p2 removed entirely
        assert_eq!(pop.population_for_player(p1), 2);
        assert_eq!(pop.population_for_player(p2), 0);
    }

    #[test]
    fn test_max_pop_one_three_players_unequal() {
        // max_pop=1, p1 has 3, p2 has 2, p3 has 1
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let p3 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 1, &[(p1, 3), (p2, 2), (p3, 1)]);
        let mut players = vec![p1, p2, p3];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_max_pop_is_one_conflicts(&mut players, &mut pop, &mut commands);

        assert_eq!(pop.population_for_player(p1), 2);
        assert_eq!(pop.population_for_player(p2), 0);
        assert_eq!(pop.population_for_player(p3), 0);
    }

    // ========================================================================
    // is_conflict_zone tests
    // ========================================================================

    #[test]
    fn test_is_conflict_zone_single_player_no_city_no_conflict() {
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let pop = create_populated_area(&mut world, 2, &[(p1, 3)]);
        // Single player, no city → not a conflict zone (even if over max)
        assert!(!pop.is_conflict_zone(false));
    }

    #[test]
    fn test_is_conflict_zone_single_player_with_city() {
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let pop = create_populated_area(&mut world, 2, &[(p1, 1)]);
        // Single player with city and population → IS a conflict zone
        assert!(pop.is_conflict_zone(true));
    }

    #[test]
    fn test_is_conflict_zone_two_players_over_max() {
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let pop = create_populated_area(&mut world, 2, &[(p1, 2), (p2, 1)]);
        assert!(pop.is_conflict_zone(false));
    }

    #[test]
    fn test_is_conflict_zone_two_players_at_max() {
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let pop = create_populated_area(&mut world, 4, &[(p1, 2), (p2, 2)]);
        // Two players but total == max → NOT a conflict zone
        assert!(!pop.is_conflict_zone(false));
    }

    #[test]
    fn test_is_conflict_zone_empty_area_with_city() {
        let pop = Population::new(2);
        // No population but has city → no conflict (has_population is false)
        assert!(!pop.is_conflict_zone(true));
    }

    // ========================================================================
    // Regression: ensure handle_unequal_lengths doesn't infinite loop
    // ========================================================================

    #[test]
    fn test_unequal_lengths_many_players_terminates() {
        // Stress test: 5 players with various token counts in a small area
        let mut world = setup_world();
        let players_data: Vec<Entity> = (0..5).map(|_| world.spawn_empty().id()).collect();
        let token_counts = [(players_data[0], 5), (players_data[1], 4), (players_data[2], 3), (players_data[3], 2), (players_data[4], 1)];
        let mut pop = create_populated_area(&mut world, 3, &token_counts);
        let mut players = players_data.clone();

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_unequal_lengths(&mut players, &mut pop, &mut commands);

        assert!(
            pop.total_population() <= 3 || pop.number_of_players() <= 1,
            "Should resolve: total_pop={}, num_players={}",
            pop.total_population(),
            pop.number_of_players()
        );
    }

    #[test]
    fn test_all_equal_two_players_each_with_1_token_max_pop_1() {
        // Edge: max_pop=1, 2 players each with 1 token, all_lengths_equal=true
        // must_remove=1, token_rounds=1 (1*2=2 >= 1), each loses 1
        // Both players eliminated
        let mut world = setup_world();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let mut pop = create_populated_area(&mut world, 1, &[(p1, 1), (p2, 1)]);
        let players = vec![p1, p2];

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        handle_all_lengths_equal(&players, &mut pop, &mut commands);

        // Both players should be eliminated (each had 1, lost 1)
        assert_eq!(pop.total_population(), 0);
    }
}
