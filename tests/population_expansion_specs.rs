mod common;

use crate::common::{create_area, setup_bevy_app, setup_player};
use adv_civ::civilization::game_moves::events::RecalculatePlayerMoves;
use adv_civ::civilization::game_moves::systems::recalculate_pop_exp_moves_for_player;
use adv_civ::civilization::components::prelude::*;
use adv_civ::civilization::enums::general_enums::GameFaction;
use bevy::app::Update;
use bevy::prelude::Events;

#[test]
fn calculate_game_moves_in_population_expansion() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_pop_exp_moves_for_player)
        ;
        app
    });

    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

    let area_one = create_area(&mut app, "Egypt", 1);
    let area_two = create_area(&mut app, "Thrace", 1);
    let area_three = create_area(&mut app, "Crete", 1);
    let areas = vec![area_one, area_two, area_three];
    for area in areas.iter() {
        let mut population = Population::new(4);
        let token = stock.remove_token_from_stock().unwrap();
        player_areas.add_token_to_area(*area, token);
        population.add_token_to_area(player, token);
        app
            .world_mut()
            .entity_mut(*area)
            .insert(population);
    }

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));


    let mut events = app.world_mut()
        .resource_mut::<Events<RecalculatePlayerMoves>>();

    events.send(RecalculatePlayerMoves::new(player));

    // Act
    app.update();
    // Assert
    let player_moves = app
        .world()
        .entity(player)
        .get::<AvailableMoves>();
    assert!(player_moves.is_some());
    let player_moves = player_moves.unwrap();
    assert_eq!(player_moves.moves.len(), 3);
    for (_move_index, p_move) in player_moves.moves.iter() {
        assert!(matches!(p_move, Move::PopulationExpansion(..)));
        if let Move::PopulationExpansion(pop_exp) = p_move {
            assert_eq!(pop_exp.max_tokens, 1);
        };
    }
}

#[test]
fn given_a_player_with_too_few_tokens_for_expansion_the_correct_moves_are_created() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_pop_exp_moves_for_player)
        ;
        app
    });

    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

    let area = create_area(&mut app, "Egypt", 1);
    let mut population = Population::new(4);

    let tokens_to_add = stock.remove_tokens_from_stock(2).unwrap();
    tokens_to_add.iter().for_each(|token| {
        player_areas.add_token_to_area(area, *token);
        population.add_token_to_area(player, *token);
    });

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));

    app
        .world_mut()
        .entity_mut(area)
        .insert(population);

    let mut events = app.world_mut()
        .resource_mut::<Events<RecalculatePlayerMoves>>();

    events.send(RecalculatePlayerMoves::new(player));

    // Act
    app.update();
    // Assert
    let player_moves = app
        .world()
        .entity(player)
        .get::<AvailableMoves>();
    assert!(player_moves.is_some());
    let player_moves = player_moves.unwrap();
    assert_eq!(player_moves.moves.len(), 1);
    let (_index, first_move) = player_moves.moves.iter().next().unwrap();
    assert!(matches!(*first_move, Move::PopulationExpansion(..)));
    if let Move::PopulationExpansion(pop_exp_move) = *first_move {
        assert_eq!(pop_exp_move.max_tokens, 2);
        assert_eq!(pop_exp_move.area, area);
    };
}