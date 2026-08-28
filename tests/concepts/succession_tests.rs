use crate::{setup_bevy_app, setup_player};
use adv_civ::GameActivity;
use adv_civ::civilization::{
    AstPosition, AstTrack, GameFaction, GameInfoAndStuff, PlayerCities, RoundLimit, RoundSummary,
    advance_succession_markers,
};
use bevy::app::Update;
use bevy::prelude::NextState::Pending;
use bevy::prelude::{Entity, NextState};

fn setup_app() -> bevy::prelude::App {
    setup_bevy_app(|mut app| {
        app.init_resource::<AstTrack>();
        app.init_resource::<GameInfoAndStuff>();
        app.init_resource::<RoundLimit>();
        app.init_resource::<RoundSummary>();
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
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(0));

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
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(4));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 4, "should be frozen at 4 without 2 cities");
}

#[test]
fn player_enters_early_bronze_with_two_cities() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 2);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(4));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(
        pos.space, 5,
        "should advance into Early Bronze with 2 cities"
    );
}

#[test]
fn player_retreats_when_no_cities() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    // No cities
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(5));

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
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(0));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(
        pos.space, 1,
        "city-less marker advances through the Stone Age"
    );
}

#[test]
fn player_with_no_cities_freezes_at_bronze_threshold() {
    // At space 4 (Stone Age) the next space enters Early Bronze (needs 2 cities).
    // With no cities the marker cannot advance, but it is in the Stone Age so it
    // does not retreat either — it freezes.
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(4));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(
        pos.space, 4,
        "city-less marker freezes at the Bronze threshold"
    );
}

#[test]
fn finished_marker_stays_at_finish() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 9);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(16));

    app.update();

    let pos = app.world().entity(player).get::<AstPosition>().unwrap();
    assert_eq!(pos.space, 16, "marker never advances past FINISH (16)");
}

#[test]
fn game_ends_when_a_marker_is_already_at_finish() {
    // Rule 34.1A: reaching a finish square ends the game. The end-of-round
    // check runs on every player's *current* position after the per-player
    // advance/retreat/freeze loop, so a player already sitting on FINISH (16)
    // trips it immediately, with no need to satisfy Late Iron's card
    // requirements to get there in this same tick.
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 9);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(16));

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::GameOver)));
}

#[test]
fn game_continues_when_no_marker_is_at_finish() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(0));

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::CollectTaxes)));
}

#[test]
fn game_ends_at_configured_round_limit_even_without_finish() {
    // Rule 34.1B: a predetermined time limit also ends the game, independent
    // of anyone's A.S.T. position.
    let mut app = setup_app();
    app.insert_resource(RoundLimit(Some(5)));
    app.insert_resource(GameInfoAndStuff {
        round: 5,
        ..Default::default()
    });
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(2));

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::GameOver)));
}

#[test]
fn game_continues_below_the_configured_round_limit() {
    let mut app = setup_app();
    app.insert_resource(RoundLimit(Some(5)));
    app.insert_resource(GameInfoAndStuff {
        round: 4,
        ..Default::default()
    });
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(2));

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::CollectTaxes)));
}

#[test]
fn no_round_limit_configured_never_ends_the_game_early() {
    // RoundLimit's default (None) — the resource is present (set up by
    // SuccessionPlugin) but unset; only the A.S.T. condition should apply.
    let mut app = setup_app();
    app.insert_resource(GameInfoAndStuff {
        round: 9_999,
        ..Default::default()
    });
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    add_cities(&mut app, player, 1);
    app.world_mut()
        .entity_mut(player)
        .insert(AstPosition::new(2));

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::CollectTaxes)));
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
