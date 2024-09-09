mod common;

use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Events, Name};
use bevy::state::app::StatesPlugin;
use bevy_console::PrintConsoleLine;
use bevy_game::civilization::general::general_components::{GameArea, LandPassage, PlayerAreas, Population};
use bevy_game::civilization::movement::movement_events::MoveTokenFromAreaToAreaCommand;
use bevy_game::{GameActivity, GameState};
use bevy_game::civilization::general::general_enums::GameFaction;
use bevy_game::civilization::movement::movement_components::{MoveableTokens, NeedsTocalculateMoves};
use bevy_game::civilization::movement::movement_systems::move_tokens_from_area_to_area;
use crate::common::setup_player;

fn setup_app() -> App {
    let mut app = App::new();
    app
        .add_plugins(
            StatesPlugin
        )
        .add_event::<MoveTokenFromAreaToAreaCommand>()
        .add_event::<PrintConsoleLine>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, move_tokens_from_area_to_area);
    app
}

#[test]
fn moving_token_to_area_adds_area_to_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..3).collect());
    population.total_population = 3;

    let from_area = app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea::default(),
            LandPassage::default(),
            population
        )
    ).id();

    let to_area = app.world_mut().spawn(
        (
            Name::new("crete"),
            GameArea::default(),
            LandPassage::default(),
            Population::new(3)
        )
    ).id();
    let mut events = app.world_mut()
        .resource_mut::<Events<MoveTokenFromAreaToAreaCommand>>();

    events.send(MoveTokenFromAreaToAreaCommand::new(from_area, to_area, 2, player_one));

    // Act
    app.update();
    // Assert
    assert!(app.world().entity(from_area).get::<MoveableTokens>().is_none());
    assert!(app.world().entity(from_area).get::<NeedsTocalculateMoves>().is_some());
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(player_area.contains(to_area));
}

#[test]
fn moving_all_tokens_from_area_removes_area_from_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..3).collect());
    population.total_population = 3;

    let from_area = app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea::default(),
            LandPassage::default(),
            population
        )
    ).id();

    let to_area = app.world_mut().spawn(
        (
            Name::new("crete"),
            GameArea::default(),
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
    assert!(app.world().entity(from_area).get::<MoveableTokens>().is_none());
    assert!(app.world().entity(from_area).get::<NeedsTocalculateMoves>().is_some());
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(!player_area.contains(from_area));
}