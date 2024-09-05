mod common;

use bevy::prelude::{Events, NextState, Update};
use bevy::prelude::NextState::Pending;
use bevy_game::civilization::city_support::plugin::{check_city_support, CheckPlayerCitySupport};
use bevy_game::civilization::game_phases::game_activity::GameActivity;
use bevy_game::civilization::general::components::BuiltCity;
use common::{setup_player, setup_bevy_app};
use crate::common::create_area;

#[test]
fn given_no_cities_next_state_is_set() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, check_city_support)
        ;
        app
    });

    setup_player(&mut app, "Player 1");
    create_area(&mut app, "Egypt");

    app.update();

    let state = app.world().get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, Pending(GameActivity::PopulationExpansion)));
}

#[test]
fn given_one_city_event_sent_for_player() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, check_city_support)
        ;
        app
    });

    let (player, tokens, mut city_tokens) = setup_player(&mut app, "Player 1");
 let area = create_area(&mut app, "Egypt");
    app
        .world_mut()
        .entity_mut(area)
        .insert(BuiltCity { city: city_tokens.pop().unwrap(), player });
    

    app.update();
    let events = app.world()
        .resource::<Events<CheckPlayerCitySupport>>();

    let mut reader = events.get_reader();

    // Assert
    assert!(!reader.is_empty(&events));
    assert_eq!(reader.len(&events), 1);
    let s = reader.read(events).next().unwrap();
    assert_eq!(s.player, player);
}