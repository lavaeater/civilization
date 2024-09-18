mod common;
use bevy::prelude::{Events, Update};
use bevy_game::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use bevy_game::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use bevy_game::civilization::game_moves::game_moves_systems::recalculate_movement_moves_for_player;
use bevy_game::civilization::general::general_components::{LandPassage, PlayerAreas, PlayerStock, Population};
use bevy_game::civilization::general::general_enums::GameFaction;
use crate::common::{create_area, setup_bevy_app, setup_player};

#[test]
fn calculate_one_move() {
    // Arrange                    
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_movement_moves_for_player)
        ;
        app
    });

    let area_one = create_area(&mut app, "Egypt");
    let area_two = create_area(&mut app, "Thrace");
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    app
        .world_mut()
        .entity_mut(area_one)
        .insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = PlayerStock::new(47, tokens.drain(0..4).collect());

    let mut population = Population::new(4);
    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    app
        .world_mut()
        .entity_mut(area_one)
        .insert(population);

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
    assert_eq!(player_moves.moves.len(), 2);
    let first_move = player_moves.moves.get(&1).unwrap();
    assert!(matches!(first_move, Move::Movement(..)));
    if let Move::Movement(m) = first_move {
        assert_eq!(m.max_tokens, 1);
        assert_eq!(m.target, area_two);
        assert_eq!(m.source, area_one);
    };

    let last_move = player_moves.moves.get(&2).unwrap();
    assert!(matches!(last_move, Move::EndMovement));
}

#[test]
fn calculate_two_moves() {
    // Arrange                    
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_movement_moves_for_player)
        ;
        app
    });

    let area_one = create_area(&mut app, "Egypt");
    let area_two = create_area(&mut app, "Thrace");
    let area_three = create_area(&mut app, "Throgdor");
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    land_passage.add_passage(area_three);
    app
        .world_mut()
        .entity_mut(area_one)
        .insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = PlayerStock::new(47, tokens.drain(0..4).collect());

    let mut population = Population::new(4);
    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    app
        .world_mut()
        .entity_mut(area_one)
        .insert(population);

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
    let first_move = player_moves.moves.get(&1).unwrap();
    assert!(matches!(first_move, Move::Movement(..)));
    if let Move::Movement(m) = first_move {
        assert_eq!(m.max_tokens, 1);
        assert_eq!(m.target, area_two);
        assert_eq!(m.source, area_one);
    };

    let second_move = player_moves.moves.get(&2).unwrap();
    assert!(matches!(second_move, Move::Movement(..)));
    if let Move::Movement(m) = second_move {
        assert_eq!(m.max_tokens, 1);
        assert_eq!(m.target, area_three);
        assert_eq!(m.source, area_one);
    };

    let last_move = player_moves.moves.get(&3).unwrap();
    assert!(matches!(last_move, Move::EndMovement));
}


