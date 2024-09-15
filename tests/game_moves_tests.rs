mod common;

use bevy::prelude::{Events, Update};
use bevy_game::civilization::game_moves::game_moves_plugin::{recalculate_pop_exp_moves_for_player, AvailableMoves, Move, RecalculatePlayerMoves};
use bevy_game::civilization::general::general_components::{PlayerAreas, PlayerCities, Population, Stock};
use bevy_game::civilization::general::general_enums::GameFaction;
use crate::common::{create_area, setup_bevy_app, setup_player};

#[test]
fn given_a_player_with_too_few_tokens_for_expansion_the_corrct_moves_are_created() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_pop_exp_moves_for_player)
        ;
        app
    });

    let (player, mut tokens, city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = Stock::new(47, tokens.drain(0..4).collect());

    let area = create_area(&mut app, "Egypt");
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
    let first_move = player_moves.moves.first().unwrap();
    assert!(matches!(*first_move, Move::PopulationExpansion(..)));
    match *first_move {
        Move::PopulationExpansion(index, move_area, tokens) => {
            assert_eq!(index, 1);
            assert_eq!(tokens, 2);
            assert_eq!(move_area, area);
        }
    };
    
}