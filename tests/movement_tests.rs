mod common;

use crate::common::{setup_bevy_app, setup_player};
use adv_civ::{GameActivity, GameState};
use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Bundle, Entity, Events, Name, Transform};
use bevy::state::app::StatesPlugin;
use bevy_console::PrintConsoleLine;
use adv_civ::civilization::components::prelude::*;
use adv_civ::civilization::enums::prelude::GameFaction;
use adv_civ::civilization::events::prelude::*;
use adv_civ::civilization::systems::prelude::*;

fn setup_app() -> App {
    let mut app = App::new();
    app
        .add_plugins(
            StatesPlugin
        )
        .add_event::<MoveTokenFromAreaToAreaCommand>()
        .add_event::<RecalculatePlayerMoves>()
        .add_event::<PrintConsoleLine>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, move_tokens_from_area_to_area);
    app
}

#[test]
fn moved_tokens_get_token_has_moved_component_added() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(6);

    for token in player_one_tokens.drain(0..3).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }

    let from_area = app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            Transform::from_xyz(0.0, 0.0, 0.0)
        )
    ).id();

    let to_area = app.world_mut().spawn(
        (
            Name::new("crete"),
            GameArea::new(2),
            LandPassage::default(),
            Population::new(3),
            Transform::from_xyz(0.0, 0.0, 0.0)
        )
    ).id();
    let mut events = app.world_mut()
        .resource_mut::<Events<MoveTokenFromAreaToAreaCommand>>();

    events.send(MoveTokenFromAreaToAreaCommand::new(from_area, to_area, 2, player_one));

    // Act
    app.update();
    // Assert
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(player_area.contains(to_area));
    let tokens_in_area = player_area.tokens_for_area(to_area);
    assert!(tokens_in_area.is_some());
    let tokens_in_area = tokens_in_area.unwrap();
    assert_eq!(2, tokens_in_area.len());
    for token in tokens_in_area {
        assert!(app.world().entity(token).get::<TokenHasMoved>().is_some());
    }

    let from_pop = app.world().entity(from_area).get::<Population>();

    let from_tokens = from_pop.unwrap().tokens_for_player(&player_one);
    assert!(from_tokens.is_some());
    let from_tokens = from_tokens.unwrap();
    assert_eq!(1, from_tokens.len());
}

#[test]
fn moving_token_to_area_adds_area_to_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    for token in player_one_tokens.drain(0..3).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }

    let from_area = create_area(&mut app, "egypt", Some(population));

    let to_area = create_area(&mut app, "crete", None::<()>);
    
    let mut events = app.world_mut()
        .resource_mut::<Events<MoveTokenFromAreaToAreaCommand>>();

    events.send(MoveTokenFromAreaToAreaCommand::new(from_area, to_area, 2, player_one));

    // Act
    app.update();
    // Assert
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(player_area.contains(to_area));
}

fn create_area<T: Bundle>(app: &mut App, name: &str, components: Option<T>) -> Entity {
    let area = app.world_mut().spawn(
        (
            Name::new(name.to_string()),
            GameArea::new(1),
            LandPassage::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Population::new(3)
        )
    ).id();
    if let Some(components) = components {
        app.world_mut().entity_mut(area).insert(components);
    }
    area
}

#[test]
fn moving_all_tokens_from_area_removes_area_from_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    for token in player_one_tokens.drain(0..3).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }

    let from_area = app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population
        )
    ).id();

    let to_area = app.world_mut().spawn(
        (
            Name::new("crete"),
            GameArea::new(2),
            LandPassage::default(),
            Population::new(3)
        )
    ).id();
    let mut events = app.world_mut()
        .resource_mut::<Events<MoveTokenFromAreaToAreaCommand>>();

    events.send(MoveTokenFromAreaToAreaCommand::new(from_area, to_area, 3, player_one));

    // Act
    app.update();
    // Assert
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(!player_area.contains(from_area));
}

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

    let area_one = crate::common::create_area(&mut app, "Egypt", 1);
    let area_two = crate::common::create_area(&mut app, "Thrace", 2);
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    app
        .world_mut()
        .entity_mut(area_one)
        .insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

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

    let area_one = crate::common::create_area(&mut app, "Egypt", 1);
    let area_two = crate::common::create_area(&mut app, "Thrace", 2);
    let area_three = crate::common::create_area(&mut app, "Throgdor", 3);
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    land_passage.add_passage(area_three);
    app
        .world_mut()
        .entity_mut(area_one)
        .insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

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

    let area_one = crate::common::create_area(&mut app, "Egypt", 1);
    let area_two = crate::common::create_area(&mut app, "Thrace", 2);
    let area_three = crate::common::create_area(&mut app, "Throgdor", 3);
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    land_passage.add_passage(area_three);
    app
        .world_mut()
        .entity_mut(area_one)
        .insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

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