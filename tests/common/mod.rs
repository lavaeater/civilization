use bevy::app::App;
use bevy::prelude::Entity;
use bevy::core::Name;
use bevy_game::civilization::general::components::{CityToken, CityTokenStock, Stock, Token, Treasury};
use bevy_game::player::Player;

pub fn setup_player(mut app: App, name: impl Into<String>) -> (App, Entity, Vec<Entity>) {
    let player = app.world_mut()
        .spawn(
            (
                Player {},
                Name::new(name.into()),
                Treasury { tokens: vec![] },
            )
        ).id();

    let tokens = (0..47).map(|_| {
        app.world_mut()
            .spawn(
                (
                    Name::new("Token 1"),
                    Token::new(player))).id()
    })
        .collect::<Vec<Entity>>();

    let city_tokens = (0..9).map(|_| {
        app.world_mut()
            .spawn(
                (
                    Name::new("City 1"),
                    CityToken::new(player))).id()
    }
    )
        .collect::<Vec<Entity>>();

    app
        .world_mut()
        .entity_mut(player)
        .insert(
            (
                Stock::new(
                    47,
                    tokens.clone()),
                CityTokenStock::new(
                    9,
                    city_tokens,
                )
            )
        );
    (app, player, tokens.clone())
}