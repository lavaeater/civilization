mod common;

use crate::common::setup_player;
use adv_civ::civilization::conflict::conflict_components::UnresolvedConflict;
use adv_civ::civilization::conflict::conflict_systems::find_conflict_zones;
use adv_civ::civilization::conflict::conflict_triggers::{on_add_unresolved_city_conflict, on_add_unresolved_conflict};
use adv_civ::civilization::general::general_components_file::{GameArea, LandPassage, Population};
use adv_civ::civilization::general::general_enums::GameFaction;
use adv_civ::civilization::general::general_events::*;
use adv_civ::{GameActivity, GameState};
use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Name};
use bevy::state::app::StatesPlugin;
use bevy_console::PrintConsoleLine;
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

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) = setup_player(&mut app, "player two", GameFaction::Crete);

    let mut population = Population::new(4);

    population.player_tokens.insert(player_one, player_one_tokens.drain(0..7).collect());
    population.player_tokens.insert(player_two, player_two_tokens.drain(0..5).collect());

    let area = app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea::new(1),
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

pub struct TwoPlayerTestStruct {
    pub player_one_tokens: usize,
    pub player_two_tokens: usize,
    pub area_max_population: usize,
    pub player_one_expected: usize,
    pub player_two_expected: usize,
}

impl TwoPlayerTestStruct {
    fn expected(&self) -> usize {
        self.player_one_expected + self.player_two_expected
    }
    
    fn new(player_one_tokens: usize,
           player_two_tokens: usize,
           area_max_population: usize,
           player_one_expected: usize,
           player_two_expected: usize) -> Self {
        TwoPlayerTestStruct {
            player_one_tokens,
            player_two_tokens,
            area_max_population,
            player_one_expected,
            player_two_expected,
        }
    }
}

pub struct ThreePlayerTestStruct {
    pub player_one_tokens: usize,
    pub player_two_tokens: usize,
    pub player_three_tokens: usize,
    pub area_max_population: usize,
    pub player_one_expected: usize,
    pub player_two_expected: usize,
    pub player_three_expected: usize,
}

impl ThreePlayerTestStruct {
    fn expected(&self) -> usize {
        self.player_one_expected + self.player_two_expected + self.player_three_expected
    }
    
    fn new(player_one_tokens: usize,
           player_two_tokens: usize,
           player_three_tokens: usize,
           area_max_population: usize,
           player_one_expected: usize,
           player_two_expected: usize,
           player_three_expected: usize) -> Self {
        ThreePlayerTestStruct {
            player_one_tokens,
            player_two_tokens,
            player_three_tokens,
            area_max_population,
            player_one_expected,
            player_two_expected,
            player_three_expected,
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
        // .add_systems(Update, find_conflict_zones)
        .observe(on_add_unresolved_conflict)
        .observe(on_add_unresolved_city_conflict)
    ;

    let test_cases = vec![
        TwoPlayerTestStruct::new(1, 1, 1, 0, 0),
        TwoPlayerTestStruct::new(2, 1, 1, 2, 0),
        TwoPlayerTestStruct::new(2, 2, 1, 0, 0),
        TwoPlayerTestStruct::new(2, 1, 2, 2, 0),
        TwoPlayerTestStruct::new(2, 2, 2, 1, 1),
        TwoPlayerTestStruct::new(3, 2, 2, 2, 0),
        TwoPlayerTestStruct::new(3, 2, 3, 2, 1),
        TwoPlayerTestStruct::new(3, 3, 3, 1, 1),
        TwoPlayerTestStruct::new(4, 2, 3, 3, 0),
        TwoPlayerTestStruct::new(3, 3, 4, 2, 2),
        TwoPlayerTestStruct::new(4, 1, 4, 4, 0),
        TwoPlayerTestStruct::new(4, 2, 4, 3, 1),
        TwoPlayerTestStruct::new(4, 3, 4, 3, 1),
        TwoPlayerTestStruct::new(5, 3, 4, 3, 1),
    ];

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) = setup_player(&mut app, "player two", GameFaction::Crete);

    for test_case in test_cases {
        let mut population = Population::new(test_case.area_max_population);

        population.player_tokens.insert(player_one, player_one_tokens.drain(0..test_case.player_one_tokens).collect());
        population.player_tokens.insert(player_two, player_two_tokens.drain(0..test_case.player_two_tokens).collect());

        let area = app.world_mut().spawn(
            (
                Name::new("egypt"),
                GameArea::new(1),
                LandPassage::default(),
                UnresolvedConflict,
                population
            )
        ).id();


        // Act
        app.update();
        // Assert
        let population = app.world().get::<Population>(area).unwrap();
        assert_eq!(
            population.total_population(),
            test_case.expected(),
            "Conflict resolution with p1: {}, p2: {}, max_pop: {}, exp: {}",
            test_case.player_one_tokens,
            test_case.player_two_tokens,
            test_case.area_max_population,
            test_case.expected()
        );
    }
}

#[test]
fn given_three_conflicteers_the_correct_result_is_obtained() {
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
        // .add_systems(Update, resolve_conflicts);
        .observe(on_add_unresolved_conflict)
        .observe(on_add_unresolved_city_conflict)
    ;

    let test_cases = vec![
        ThreePlayerTestStruct::new(1, 1, 1, 1, 0, 0, 0),
        ThreePlayerTestStruct::new(4, 2, 1, 3, 3, 0, 0),
        ThreePlayerTestStruct::new(4, 4, 4, 3, 1, 1, 1),
        ThreePlayerTestStruct::new(2, 5, 1, 2, 0, 4, 0),
        ThreePlayerTestStruct::new(2, 5, 1, 3, 0, 4, 0),
    ];

    let (player_one, mut player_one_tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let (player_two, mut player_two_tokens, _) = setup_player(&mut app, "player two", GameFaction::Crete);
    let (player_three, mut player_three_tokens, _) = setup_player(&mut app, "player three", GameFaction::Babylon);

    for test_case in test_cases {
        let mut population = Population::new(test_case.area_max_population);

        population.player_tokens.insert(player_one, player_one_tokens.drain(0..test_case.player_one_tokens).collect());
        population.player_tokens.insert(player_two, player_two_tokens.drain(0..test_case.player_two_tokens).collect());
        population.player_tokens.insert(player_three, player_three_tokens.drain(0..test_case.player_three_tokens).collect());

        let area = app.world_mut().spawn(
            (
                Name::new("egypt"),
                GameArea::new(1),
                LandPassage::default(),
                UnresolvedConflict,
                population
            )
        ).id();


        // Act
        app.update();
        // Assert
        let population = app.world().get::<Population>(area).unwrap();
        assert_eq!(
            population.total_population(),
            test_case.expected(),
            "Conflict resolution with p1: {}, p2: {}, p3: {}, max_pop: {}, exp: {}",
            test_case.player_one_tokens,
            test_case.player_two_tokens,
            test_case.player_three_tokens,
            test_case.area_max_population,
            test_case.expected()
        );
    }
}