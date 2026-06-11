use crate::{setup_bevy_app, setup_player};
use adv_civ::civilization::{
    advance_succession_markers, AstPosition, AstTrack, GameFaction, PlayerCities,
};
use bevy::app::Update;
use bevy::prelude::Entity;

fn setup_app() -> bevy::prelude::App {
    setup_bevy_app(|mut app| {
        app.init_resource::<AstTrack>();
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
    // START (0) → 1, still Stone Age
    app.world_mut().entity_mut(player).insert(AstPosition::new(0));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 1, "should advance from START into Stone Age");
}

#[test]
fn player_frozen_at_early_bronze_threshold_with_one_city() {
    // Space 4→5 enters Early Bronze, requires ≥2 cities
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut().entity_mut(player).insert(AstPosition::new(4));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 4, "should be frozen at 4 without 2 cities");
}

#[test]
fn player_enters_early_bronze_with_two_cities() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 2);
    app.world_mut().entity_mut(player).insert(AstPosition::new(4));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 5, "should advance into Early Bronze with 2 cities");
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
fn player_with_no_cities_still_advances_in_stone_age() {
    // Rule 33.4: the Stone Age has no city requirement, so a city-less marker
    // advances normally rather than retreating.
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    // No cities, at START
    app.world_mut().entity_mut(player).insert(AstPosition::new(0));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 1, "city-less marker advances through the Stone Age");
}

#[test]
fn player_with_no_cities_freezes_at_bronze_threshold() {
    // At space 4 (Stone Age) the next space enters Early Bronze (needs 2 cities).
    // With no cities the marker cannot advance, but it is in the Stone Age so it
    // does not retreat either — it freezes.
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(player).insert(AstPosition::new(4));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 4, "city-less marker freezes at the Bronze threshold");
}

#[test]
fn finished_marker_stays_at_finish() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 9);
    app.world_mut().entity_mut(player).insert(AstPosition::new(16));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 16, "marker never advances past FINISH (16)");
}

#[test]
fn ast_epoch_epoch_boundaries_are_correct() {
    use adv_civ::civilization::AstEpoch;
    assert_eq!(AstEpoch::for_space(0), AstEpoch::StoneAge);
    assert_eq!(AstEpoch::for_space(4), AstEpoch::StoneAge);
    assert_eq!(AstEpoch::for_space(5), AstEpoch::EarlyBronze);
    assert_eq!(AstEpoch::for_space(7), AstEpoch::EarlyBronze);
    assert_eq!(AstEpoch::for_space(8), AstEpoch::LateBronze);
    assert_eq!(AstEpoch::for_space(10), AstEpoch::LateBronze);
    assert_eq!(AstEpoch::for_space(11), AstEpoch::EarlyIron);
    assert_eq!(AstEpoch::for_space(13), AstEpoch::EarlyIron);
    assert_eq!(AstEpoch::for_space(14), AstEpoch::LateIron);
    assert_eq!(AstEpoch::for_space(16), AstEpoch::LateIron);
}
