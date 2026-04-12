use crate::{setup_bevy_app, setup_player};
use adv_civ::civilization::{advance_succession_markers, AstPosition, GameFaction, PlayerCities};
use bevy::app::Update;
use bevy::prelude::Entity;

fn setup_app() -> bevy::prelude::App {
    setup_bevy_app(|mut app| {
        app.add_systems(Update, advance_succession_markers);
        app
    })
}

/// Insert N cities into the player's PlayerCities by spawning dummy area + city entities.
fn add_cities(app: &mut bevy::prelude::App, player: Entity, count: usize) {
    let pairs: Vec<(Entity, Entity)> = (0..count)
        .map(|_| {
            let area = app.world_mut().spawn_empty().id();
            let city = app.world_mut().spawn_empty().id();
            (area, city)
        })
        .collect();
    let world = app.world_mut();
    let mut entity_mut = world.entity_mut(player);
    let mut cities = entity_mut.get_mut::<PlayerCities>().unwrap();
    for (area, city) in pairs {
        cities.areas_and_cities.insert(area, city);
    }
}

#[test]
fn player_advances_in_stone_age_with_one_city() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut().entity_mut(player).insert(AstPosition::new(1));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 2, "should advance from 1→2 in Stone Age");
}

#[test]
fn player_frozen_at_early_bronze_threshold_with_one_city() {
    // Space 3→4 enters Early Bronze, requires ≥2 cities
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut().entity_mut(player).insert(AstPosition::new(3));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 3, "should be frozen at 3 without 2 cities");
}

#[test]
fn player_enters_early_bronze_with_two_cities() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 2);
    app.world_mut().entity_mut(player).insert(AstPosition::new(3));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 4, "should advance into Early Bronze with 2 cities");
}

#[test]
fn player_retreats_when_no_cities() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    // No cities
    app.world_mut().entity_mut(player).insert(AstPosition::new(5));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 4, "should retreat one space with no cities");
}

#[test]
fn player_does_not_retreat_below_space_one() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    // No cities, already at space 1
    app.world_mut().entity_mut(player).insert(AstPosition::new(1));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 1, "should not go below space 1");
}

#[test]
fn ast_epoch_epoch_boundaries_are_correct() {
    use adv_civ::civilization::{AstEpoch};
    assert_eq!(AstEpoch::for_space(1), AstEpoch::StoneAge);
    assert_eq!(AstEpoch::for_space(3), AstEpoch::StoneAge);
    assert_eq!(AstEpoch::for_space(4), AstEpoch::EarlyBronze);
    assert_eq!(AstEpoch::for_space(6), AstEpoch::EarlyBronze);
    assert_eq!(AstEpoch::for_space(7), AstEpoch::LateBronze);
    assert_eq!(AstEpoch::for_space(9), AstEpoch::LateBronze);
    assert_eq!(AstEpoch::for_space(10), AstEpoch::EarlyIron);
    assert_eq!(AstEpoch::for_space(12), AstEpoch::EarlyIron);
    assert_eq!(AstEpoch::for_space(13), AstEpoch::LateIron);
    assert_eq!(AstEpoch::for_space(20), AstEpoch::LateIron);
}
