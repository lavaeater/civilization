mod common;

use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Entity, Name};
use bevy::state::app::StatesPlugin;
use bevy_console::PrintConsoleLine;
use bevy_game::civilization::conflict::conflict_components::UnresolvedConflict;
use bevy_game::civilization::general::general_components::{GameArea, LandPassage, Population};
use bevy_game::civilization::general::general_events::*;
use bevy_game::{GameActivity, GameState};
use bevy_game::civilization::conflict::conflict_systems::find_conflict_zones;
use bevy_game::civilization::general::general_enums::GameFaction;
use crate::common::setup_player;

/****************************************************
Test for the find_conflict_zones system
Given two players that have tokens in an area,
when the system is run, that area should have a component
added indicating that it has a conflict.
*****************************************************/

#[test]
fn given_two_players_no_keys_are_left_behind() {
    // Arrange
    let mut app = App::new();
    app
        .add_plugins(
            StatesPlugin,
        )
        .add_event::<ReturnTokenToStock>()
        .add_event::<PrintConsoleLine>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, find_conflict_zones);

    let player_one: Entity;
    let mut player_one_tokens: Vec<Entity>;
    (player_one, player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let player_two: Entity;
    let mut player_two_tokens: Vec<Entity>;
    (player_two, player_two_tokens, _) = setup_player(&mut  app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..7).collect());
    population.player_tokens.insert(player_two, player_two_tokens.drain(0..5).collect());
    population.total_population = 12;

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
    // Assert
    assert!(app.world().get::<UnresolvedConflict>(area).is_some());
}