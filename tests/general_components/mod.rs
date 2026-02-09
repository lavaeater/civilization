use adv_civ::civilization::components::Population;
use adv_civ::test_utils::create_test_entity as create_entity;

#[test]
fn test_new_population() {
    let max_population = 100;
    let population = Population::new(max_population);
    assert_eq!(population.max_population, max_population);
    assert!(population.player_tokens().is_empty());
}

#[test]
fn test_add_token_to_area() {
    let mut population = Population::new(10);
    let player = create_entity();
    let token = create_entity();

    population.add_token_to_area(player, token);
    assert!(population.player_tokens().contains_key(&player));
    assert!(
        population
            .player_tokens()
            .get(&player)
            .unwrap()
            .contains(&token)
    );
}

#[test]
fn test_remove_token_from_area() {
    let mut population = Population::new(10);
    let player = create_entity();
    let token = create_entity();

    population.add_token_to_area(player, token);
    population.remove_token_from_area(player, token);

    assert!(population.tokens_for_player(&player).is_none());
    assert!(!population.has_player(&player));
}

#[test]
fn test_has_more_than_one_player() {
    let mut population = Population::new(10);
    let player1 = create_entity();
    let player2 = create_entity();
    let token = create_entity();
    let token2 = create_entity();

    population.add_token_to_area(player1, token);
    assert!(!population.has_more_than_one_player());

    population.add_token_to_area(player2, token2);
    assert!(population.has_more_than_one_player());
}

#[test]
fn test_has_other_players() {
    let mut population = Population::new(10);
    let player1 = create_entity();
    let player2 = create_entity();

    population.add_token_to_area(player1, create_entity());
    assert!(!population.has_other_players(&player1));

    population.add_token_to_area(player2, create_entity());
    assert!(population.has_other_players(&player1));
}

#[test]
fn test_all_lengths_equal() {
    let mut population = Population::new(10);
    let player1 = create_entity();
    let player2 = create_entity();
    let token1 = create_entity();
    let token2 = create_entity();

    population.add_token_to_area(player1, token1);
    population.add_token_to_area(player2, token2);

    assert!(population.all_lengths_equal());

    let token3 = create_entity();
    population.add_token_to_area(player2, token3);

    assert!(!population.all_lengths_equal());
}

#[test]
fn test_remove_surplus() {
    let mut population = Population::new(2);
    let player = create_entity();
    let token1 = create_entity();
    let token2 = create_entity();
    let token3 = create_entity();

    population.add_token_to_area(player, token1);
    population.add_token_to_area(player, token2);
    population.add_token_to_area(player, token3);

    let surplus = population.remove_surplus();
    assert_eq!(surplus.len(), 1); // Since the max_population is 2, 1 token should be removed.
    assert_eq!(population.total_population(), 2);
}

#[test]
fn test_remove_all_tokens() {
    let mut population = Population::new(10);
    let player1 = create_entity();
    let player2 = create_entity();
    let token1 = create_entity();
    let token2 = create_entity();

    population.add_token_to_area(player1, token1);
    population.add_token_to_area(player2, token2);

    let all_tokens = population.remove_all_tokens();
    assert!(all_tokens.contains(&token1));
    assert!(all_tokens.contains(&token2));
    assert!(population.player_tokens().is_empty());
}

#[test]
fn test_remove_all_tokens_for_player() {
    let mut population = Population::new(10);
    let player = create_entity();
    let token1 = create_entity();
    let token2 = create_entity();

    population.add_token_to_area(player, token1);
    population.add_token_to_area(player, token2);

    let tokens = population.remove_all_tokens_for_player(&player);
    assert!(tokens.contains(&token1));
    assert!(tokens.contains(&token2));
    assert!(!population.has_player(&player));
}

#[test]
fn test_remove_all_but_n_tokens() {
    let mut population = Population::new(10);
    let player = create_entity();
    let token1 = create_entity();
    let token2 = create_entity();
    let token3 = create_entity();
    let token4 = create_entity();

    population.add_token_to_area(player, token1);
    population.add_token_to_area(player, token2);
    population.add_token_to_area(player, token3);
    population.add_token_to_area(player, token4);

    let removed_tokens = population.remove_all_but_n_tokens(&player, 2).unwrap();
    assert!(
        removed_tokens.contains(&token1)
            || removed_tokens.contains(&token2)
            || removed_tokens.contains(&token3)
            || removed_tokens.contains(&token4)
    );
    assert_eq!(population.population_for_player(player), 2);
}

#[test]
fn test_is_conflict_zone() {
    let mut population = Population::new(2);
    let player1 = create_entity();
    let player2 = create_entity();

    population.add_token_to_area(player1, create_entity());
    population.add_token_to_area(player1, create_entity());
    population.add_token_to_area(player2, create_entity());

    assert!(population.is_conflict_zone(false));
    assert!(population.is_conflict_zone(true));
}
