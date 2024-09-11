mod common;

use bevy::prelude::{Events, NextState, Update};
use bevy::prelude::NextState::Pending;
use bevy_game::civilization::city_support::city_support_components::{HasTooManyCities, NeedsToCheckCitySupport};
use bevy_game::civilization::city_support::city_support_events::CheckPlayerCitySupport;
use bevy_game::civilization::city_support::city_support_systems::{check_city_support_gate, check_player_city_support};
use bevy_game::civilization::general::general_components::{BuiltCity, PlayerCities};
use bevy_game::civilization::general::general_enums::GameFaction;
use bevy_game::GameActivity;
use common::{setup_bevy_app, setup_player};
use crate::common::create_area;

#[test]
fn given_no_cities_next_state_is_set() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, check_city_support_gate)
        ;
        app
    });

    setup_player(&mut app, "Player 1", GameFaction::Egypt);
    create_area(&mut app, "Egypt");

    app.update();

    let state = app.world().get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, Pending(GameActivity::PopulationExpansion)));
}

#[test]
fn given_one_city_check_component_added_to_player() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_systems(Update, check_city_support_gate)
        ;
        app
    });

    let (player, _tokens, mut city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
    let mut player_cities = PlayerCities::default();
    
    let city_token = city_tokens.pop().unwrap();
    
    
    let area = create_area(&mut app, "Egypt");
    app
        .world_mut()
        .entity_mut(area)
        .insert(BuiltCity { city: city_token, player });
    
    player_cities.build_city_in_area(area, city_token);
    app
        .world_mut()
        .entity_mut(player)
        .insert(player_cities);

    app.update();

    // Assert
    assert!(app.world_mut().entity(player).contains::<NeedsToCheckCitySupport>());
}

#[test]
fn given_one_city_no_support_too_many_cities_component_added() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_systems(Update, check_player_city_support)
        ;
        app
    });

    let (player, _tokens, mut city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
    let mut player_cities = PlayerCities::default();

    let city_token = city_tokens.pop().unwrap();

    let area = create_area(&mut app, "Egypt");
    app
        .world_mut()
        .entity_mut(area)
        .insert(BuiltCity { city: city_token, player });

    player_cities.build_city_in_area(area, city_token);
    app
        .world_mut()
        .entity_mut(player)
        .insert((player_cities, NeedsToCheckCitySupport::default()));

    app.update();
    
    // Assert
    assert!(app.world_mut().entity(player).contains::<HasTooManyCities>());
    let too_many = app.world_mut().entity(player).get::<HasTooManyCities>().unwrap();
    assert_eq!(too_many.surplus_count, 1);
    assert_eq!(too_many.needed_tokens, 2);
}