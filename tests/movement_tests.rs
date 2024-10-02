mod common;

use crate::common::setup_player;
use adv_civ::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use adv_civ::civilization::general::general_components::{GameArea, LandPassage, PlayerAreas, Population};
use adv_civ::civilization::general::general_enums::GameFaction;
use adv_civ::civilization::movement::movement_components::TokenHasMoved;
use adv_civ::civilization::movement::movement_events::MoveTokenFromAreaToAreaCommand;
use adv_civ::civilization::movement::movement_systems::move_tokens_from_area_to_area;
use adv_civ::{GameActivity, GameState};
use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Bundle, Entity, Events, Name, Transform};
use bevy::state::app::StatesPlugin;
use bevy_console::PrintConsoleLine;

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

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..3).collect());

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

    let from_tokens = from_pop.unwrap().player_tokens.get(&player_one);
    assert!(from_tokens.is_some());
    let from_tokens = from_tokens.unwrap();
    assert_eq!(1, from_tokens.len());
    // for token in from_tokens {
    //     assert!(app.world().entity(token).get::<TokenHasMoved>().is_none()); 
    // }
}

#[test]
fn moving_token_to_area_adds_area_to_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..3).collect());

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

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..3).collect());

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