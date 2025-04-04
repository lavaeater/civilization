
use adv_civ::GameActivity;
use bevy::prelude::NextState::Pending;
use bevy::prelude::{Events, NextState, Update};
use adv_civ::civilization::enums::prelude::*;
use adv_civ::civilization::events::prelude::*;
use adv_civ::civilization::systems::prelude::*;
use adv_civ::civilization::components::prelude::*;
use crate::{create_area, setup_bevy_app, setup_player};

#[test]
fn given_no_cities_next_state_is_set() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, start_check_city_support)
        ;
        app
    });

    setup_player(&mut app, "Player 1", GameFaction::Egypt);
    create_area(&mut app, "Egypt",1);

    app.update();

    let state = app.world().get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, Pending(GameActivity::AcquireTradeCards)));
}

#[test]
fn given_one_city_check_component_added_to_player() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_systems(Update, start_check_city_support)
        ;
        app
    });

    let (player, _tokens, mut city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
    let mut player_cities = PlayerCities::default();

    let city_token = city_tokens.pop().unwrap();


    let area = create_area(&mut app, "Egypt",1);
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

    let area = create_area(&mut app, "Egypt",1);
    app
        .world_mut()
        .entity_mut(area)
        .insert(BuiltCity { city: city_token, player });

    player_cities.build_city_in_area(area, city_token);
    app
        .world_mut()
        .entity_mut(player)
        .insert((player_cities, NeedsToCheckCitySupport));

    app.update();

    // Assert
    assert!(app.world_mut().entity(player).contains::<HasTooManyCities>());
    let too_many = app.world_mut().entity(player).get::<HasTooManyCities>().unwrap();
    assert_eq!(too_many.surplus_count, 1);
    assert_eq!(too_many.needed_tokens, 2);
}

#[test]
fn given_a_city_to_eliminate_the_correct_things_happen() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<EliminateCity>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
            .add_systems(Update, eliminate_city)
        ;
        app
    });

    let (player, _tokens, mut city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
    let mut player_cities = PlayerCities::default();

    let city_token = city_tokens.pop().unwrap();

    let area = create_area(&mut app, "Egypt",1);
    app
        .world_mut()
        .entity_mut(area)
        .insert((BuiltCity { city: city_token, player },
        Population::new(4)));
    

    player_cities.build_city_in_area(area, city_token);
    app
        .world_mut()
        .entity_mut(player)
        .insert((player_cities, HasTooManyCities::new(1, 2)));

    let mut events = app.world_mut()
        .resource_mut::<Events<EliminateCity>>();

    events.send(EliminateCity::new(player, city_token, area, false));

    // Act
    app.update();

    // Assert
    let events = app.world_mut().resource::<Events<EliminateCity>>();
    let cursor = events.get_cursor();
    assert!(!cursor.is_empty(events));
    app.world_mut().entity(player).contains::<NeedsToCheckCitySupport>();
    assert!(app.world_mut().entity(player).get::<PlayerCities>().unwrap().has_no_cities());
    let events = app.world_mut().resource::<Events<MoveTokensFromStockToAreaCommand>>();
    let cursor = events.get_cursor();
    assert!(!cursor.is_empty(events));
}