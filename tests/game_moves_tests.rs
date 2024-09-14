mod common;

use bevy::prelude::{Events, Update};
use bevy_game::civilization::game_moves::game_moves_plugin::{recalculate_pop_exp_moves_for_player, RecalculatePlayerMoves};
use bevy_game::civilization::general::general_components::{PlayerAreas, PlayerCities, Population, Stock};
use bevy_game::civilization::general::general_enums::GameFaction;
use crate::common::{create_area, setup_bevy_app, setup_player};

#[test]
fn given_a_city_to_elimate_the_correct_things_happen() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_pop_exp_moves_for_player)
        ;
        app
    });

    let (player, tokens, mut city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = Stock::new(47, tokens.clone());

    let mut area = create_area(&mut app, "Egypt");

    let token = stock.remove_tokens_from_stock(1).unwrap().pop().unwrap();


    player_areas.add_token_to_area(area, token.clone());

    let mut population = Population::new(4);
    population.add_token_to_area(player, token);

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));

    app
        .world_mut()
        .entity_mut(area)
        .insert(population);

    let mut events = app.world_mut()
        .resource_mut::<Events<RecalculatePlayerMoves>>();

    events.send(RecalculatePlayerMoves::new(player));

    // Act
    app.update();

    // Assert
}