use bevy::app::App;
use bevy::core::Name;
use bevy::prelude::{AppExtStates, Entity};
use bevy::state::app::StatesPlugin;
use bevy_game::civilization::census::census_components::Census;
use bevy_game::civilization::general::general_components::{CityToken, CityTokenStock, Faction, GameArea, LandPassage, PlayerAreas, PlayerCities, PlayerStock, Token, Treasury};
use bevy_game::civilization::general::general_enums::GameFaction;
use bevy_game::player::Player;
use bevy_game::{GameActivity, GameState};

/*
Make sure to update this to mirror the method in
the actual game so that we have the correct components etc.
 */
#[cfg(test)]
#[allow(dead_code)]
pub fn setup_player(app: &mut App, name: impl Into<String>, faction: GameFaction) -> (Entity, Vec<Entity>, Vec<Entity>) {
    let player = app.world_mut()
        .spawn(
            (
                Player {},
                Name::new(name.into()),
                Treasury::default(),
                Census::default(),
                Faction { faction },
                PlayerAreas::default(),
                PlayerCities::default()
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
                PlayerStock::new(
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

#[cfg(test)]
#[allow(dead_code)]
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

#[cfg(test)]
#[allow(dead_code)]
pub fn create_area(app: &mut App, name: impl Into<String>) -> Entity {
    let area = app.world_mut().spawn(
        (
            Name::new(name.into()),
            GameArea {},
            LandPassage::default(),
        )
    ).id();
    area
}
