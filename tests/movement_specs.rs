mod common;
use crate::common::{create_area, setup_bevy_app, setup_player};
use adv_civ::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use adv_civ::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use adv_civ::civilization::game_moves::game_moves_systems::recalculate_movement_moves_for_player;
use adv_civ::civilization::general::general_components::{LandPassage, PlayerAreas, PlayerStock, Population};
use adv_civ::civilization::general::general_enums::GameFaction;
use adv_civ::civilization::movement::movement_events::PlayerMovementEnded;
use bevy::prelude::{Events, Update};

#[test]
fn calculate_one_move() {
    // Arrange                    
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_event::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player)
        ;
        app
    });

    let area_one = create_area(&mut app, "Egypt",1);
    let area_two = create_area(&mut app, "Thrace",2);
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

    app
        .world_mut()
        .entity_mut(area_two)
        .insert(Population::new(3));

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
            .add_event::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player)
        ;
        app
    });

    let area_one = create_area(&mut app, "Egypt",1);
    let area_two = create_area(&mut app, "Thrace",2);
    let area_three = create_area(&mut app, "Throgdor",3);
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

    app
        .world_mut()
        .entity_mut(area_two)
        .insert(Population::new(3));

    app
        .world_mut()
        .entity_mut(area_three)
        .insert(Population::new(2));

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

#[test]
fn calculate_moves_after_having_moved() {
    // Arrange                    
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_event::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player)
        ;
        app
    });

    let area_one = create_area(&mut app, "Egypt",1);
    let area_two = create_area(&mut app, "Thrace",2);
    let area_three = create_area(&mut app, "Throgdor",3);
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

    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);
    
    

    app
        .world_mut()
        .entity_mut(area_one)
        .insert(population);

    app
        .world_mut()
        .entity_mut(area_two)
        .insert(Population::new(3));

    app
        .world_mut()
        .entity_mut(area_three)
        .insert(Population::new(2));

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
        assert_eq!(m.max_tokens, 3);
        assert_eq!(m.target, area_two);
        assert_eq!(m.source, area_one);
    };

    let second_move = player_moves.moves.get(&2).unwrap();
    assert!(matches!(second_move, Move::Movement(..)));
    if let Move::Movement(m) = second_move {
        assert_eq!(m.max_tokens, 3);
        assert_eq!(m.target, area_three);
        assert_eq!(m.source, area_one);
    };

    let last_move = player_moves.moves.get(&3).unwrap();
    assert!(matches!(last_move, Move::EndMovement));
}


