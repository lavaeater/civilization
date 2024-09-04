mod common;

use bevy::prelude::{AppExtStates, Entity, Events, Update};
use bevy_game::civilization::general::components::*;
use bevy_game::civilization::general::events::*;
use bevy_game::civilization::remove_surplus::systems::remove_surplus_population;
use crate::common::{create_area_with_population, setup_bevy_app, setup_player};

#[test]
fn given_one_player_events_are_sent() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_event::<ReturnTokenToStock>()
            .add_systems(Update, remove_surplus_population);
        app
    });

    let player: Entity;
    let mut tokens: Vec<Entity>;
    (app, player, tokens) = setup_player(app, "player one");

    let mut population = Population::new(4);

    population.player_tokens.insert(player, tokens.drain(0..7).collect());
    population.total_population = 7;

    let area = create_area_with_population(&mut app, population);

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

