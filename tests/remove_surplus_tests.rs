mod common;

use bevy::prelude::{Entity, Events, Update};
use bevy_game::civilization::general::general_components::*;
use bevy_game::civilization::general::general_enums::GameFaction;
use bevy_game::civilization::general::general_events::*;
use bevy_game::civilization::remove_surplus::remove_surplus_systems::remove_surplus_population;
use crate::common::{create_area, setup_bevy_app, setup_player};

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
    (player, tokens, _) = setup_player(&mut  app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    population.player_tokens.insert(player, tokens.drain(0..7).collect());

    let area = create_area(&mut app, "Egypt", 1);

    app.world_mut().entity_mut(area)
        .insert(population);
    
    // Act
    app.update();
    let events = app.world()
        .resource::<Events<ReturnTokenToStock>>();

    let reader = events.get_reader();

    // Assert
    assert!(app.world().get::<Population>(area).is_some());
    let population = app.world().get::<Population>(area).unwrap();

    assert_eq!(population.total_population(), population.max_population);
    assert!(!reader.is_empty(&events));
    assert_eq!(reader.len(&events), 3);
}

