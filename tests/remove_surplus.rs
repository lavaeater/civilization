mod common;

use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Entity, Events, Name};
use bevy::state::app::StatesPlugin;
use bevy_game::civilization::game_phases::game_activity::*;
use bevy_game::civilization::general::components::*;
use bevy_game::civilization::general::events::*;
use bevy_game::civilization::remove_surplus::systems::*;
use bevy_game::GameState;
use crate::common::setup_player;

#[test]
fn given_one_player_events_are_sent() {
    // Arrange
    let mut app = App::new();
    app
        .add_plugins(
            StatesPlugin,
        )
        .add_event::<ReturnTokenToStock>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, remove_surplus_population);

    let player: Entity;
    let mut tokens: Vec<Entity>;
    (app, player, tokens) = setup_player(app, "player one");

    let mut population = Population::new(4);

    population.player_tokens.insert(player, tokens.drain(0..7).collect());
    population.total_population = 7;

    let area = app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea {},
            LandPassage::default(),
            population
        )
    ).id();
    

    // Act
    app.update();
    let events = app.world()
        .resource::<Events<ReturnTokenToStock>>();

    let reader = events.get_reader();

    // Assert
    assert!(app.world().get::<Population>(area).is_some());
    let population = app.world().get::<Population>(area).unwrap();

    assert_eq!(population.total_population, population.max_population);
    assert!(!reader.is_empty(&events));
    assert_eq!(reader.len(&events), 3);
}