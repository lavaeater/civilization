use bevy::app::App;
use bevy::core::Name;
use bevy::prelude::{AppExtStates, Entity};
use bevy::state::app::StatesPlugin;
use bevy_game::civilization::game_phases::game_activity::GameActivity;
use bevy_game::civilization::general::components::{CityToken, CityTokenStock, GameArea, LandPassage, Population, Stock, Token, Treasury};
use bevy_game::player::Player;
use bevy_game::GameState;

pub fn setup_player(app: &mut App, name: impl Into<String>) -> (Entity, Vec<Entity>, Vec<Entity>) {
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
                    city_tokens.clone(),
                )
            )
        );
    (player, tokens, city_tokens)
}

pub fn setup_bevy_app(app_builder: fn(App)->App) -> App {
    let mut app = App::new();
    app
        .add_plugins(
            StatesPlugin,
        )
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>();
    
    app_builder(app)
}

pub fn create_area(app: &mut App, name: impl Into<String>) -> Entity {
    app.world_mut().spawn(
        (
            Name::new(name.into()),
            GameArea {},
            LandPassage::default(),
        )
    ).id()
}

pub fn create_area_with_population(app: &mut App, population: Population) -> Entity {
    app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea {},
            LandPassage::default(),
            population
        )
    ).id()
}