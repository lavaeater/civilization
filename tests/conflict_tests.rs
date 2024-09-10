mod common;

use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Entity, Name};
use bevy::state::app::StatesPlugin;
use bevy_console::PrintConsoleLine;
use bevy_game::civilization::conflict::conflict_components::UnresolvedConflict;
use bevy_game::civilization::general::general_components::{GameArea, LandPassage, Population};
use bevy_game::civilization::general::general_events::*;
use bevy_game::{GameActivity, GameState};
use bevy_game::civilization::conflict::conflict_systems::{find_conflict_zones, resolve_conflicts};
use bevy_game::civilization::general::general_enums::GameFaction;
use crate::common::setup_player;

/****************************************************
Test for the find_conflict_zones system
Given two players that have tokens in an area,
when the system is run, that area should have a component
added indicating that it has a conflict.
*****************************************************/

#[test]
fn given_two_players_in_an_area_with_too_much_population_area_is_marked_as_conflict_zone() {
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
/*
    mut conflict_zones: Query<(Entity, &Name, &mut Population), With<UnresolvedConflict>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands,
    name_query: Query<&Name>,
    mut write_line: EventWriter<PrintConsoleLine>,
 */

pub struct ConflictTestStruct {
    pub player_one_tokens: usize,
    pub player_two_tokens: usize,
    pub area_max_population: usize,
    pub player_one_expected: usize,
    pub player_two_expected: usize
}

impl ConflictTestStruct {
    fn new(player_one_tokens: usize, 
           player_two_tokens: usize, 
           area_max_population: usize,
           player_one_expected: usize,
           player_two_expected: usize) -> Self {
        
        ConflictTestStruct {
            player_one_tokens,
            player_two_tokens,
            area_max_population,
            player_one_expected,
            player_two_expected
        }
    }
}

#[test]
fn when_resolving_conflicts_the_correct_result_is_obtained() {
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
        .add_systems(Update, resolve_conflicts);
    
    let test_cases = vec![
        ConflictTestStruct::new(1,1,1,0,0),
        ConflictTestStruct::new(2,1,1,2,0),
        ConflictTestStruct::new(2,1,2,2,0),
        ConflictTestStruct::new(2,2,2,1,1),
        ConflictTestStruct::new(3,2,2,2,0),
        ConflictTestStruct::new(3,2,3,2,1),
        ConflictTestStruct::new(3,3,3,1,1),
        ConflictTestStruct::new(4,2,3,3,0),
        ConflictTestStruct::new(3,3,4,2,2),
        ConflictTestStruct::new(4,1,4,4,0),
        ConflictTestStruct::new(4,2,4,3,1),
        ConflictTestStruct::new(4,3,4,3,1),
        ConflictTestStruct::new(5,3,4,3,1),
    ];

    let player_one: Entity;
    let mut player_one_tokens: Vec<Entity>;
    (player_one, player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let player_two: Entity;
    let mut player_two_tokens: Vec<Entity>;
    (player_two, player_two_tokens, _) = setup_player(&mut  app, "player two", GameFaction::Crete);

    for test_case in test_cases {
        let mut population = Population::new(test_case.area_max_population);

        population.player_tokens.insert(player_one, player_one_tokens.drain(0..test_case.player_one_tokens).collect());
        population.player_tokens.insert(player_two, player_two_tokens.drain(0..test_case.player_two_tokens).collect());

        let area = app.world_mut().spawn(
            (
                Name::new("egypt"),
                GameArea::default(),
                LandPassage::default(),
                UnresolvedConflict::default(),
                population
            )
        ).id();


        // Act
        app.update();
        // Assert
        let population = app.world().get::<Population>(area).unwrap();
        assert_eq!(
            population.total_population(), 
            test_case.player_two_expected + test_case.player_one_expected, 
            "Conflict resolution with p1: {}, p2: {}, max_pop: {}, exp: {}",
            test_case.player_one_tokens,
            test_case.player_two_tokens,
            test_case.area_max_population,
            test_case.player_one_expected + test_case.player_two_expected
        );
    }

    
}