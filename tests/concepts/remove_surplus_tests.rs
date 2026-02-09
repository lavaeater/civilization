use crate::{create_area, setup_bevy_app, setup_player};
use adv_civ::civilization::components::BuiltCity;
use adv_civ::civilization::components::Population;
use adv_civ::civilization::concepts::remove_surplus_population;
use adv_civ::civilization::enums::GameFaction;
use bevy::prelude::{Entity, Update};

#[test]
fn given_one_player_events_are_sent() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_systems(Update, remove_surplus_population);
        app
    });

    let player: Entity;
    let mut tokens: Vec<Entity>;
    (player, tokens, _) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    for token in tokens.drain(0..7).collect::<Vec<_>>() {
        population.add_token_to_area(player, token);
    }

    let area = create_area(&mut app, "Egypt", 1);

    app.world_mut().entity_mut(area).insert(population);

    // Act
    app.update();
    // Assert
    assert!(app.world().get::<Population>(area).is_some());
    let population = app.world().get::<Population>(area).unwrap();

    assert_eq!(population.total_population(), population.max_population);
}

#[test]
fn given_city_area_with_tokens_all_are_removed() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_systems(Update, remove_surplus_population);
        app
    });

    let player: Entity;
    let mut tokens: Vec<Entity>;
    let mut city_tokens: Vec<Entity>;
    (player, tokens, city_tokens) = setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    for token in tokens.drain(0..4).collect::<Vec<_>>() {
        population.add_token_to_area(player, token);
    }

    let area = create_area(&mut app, "Egypt", 1);

    app.world_mut().entity_mut(area).insert((
        population,
        BuiltCity::new(city_tokens.pop().unwrap(), player),
    ));

    // Act
    app.update();

    // Assert
    assert!(app.world().get::<Population>(area).is_some());
    let population = app.world().get::<Population>(area).unwrap();

    assert_eq!(population.total_population(), 0);
}
