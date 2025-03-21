mod general_components;
mod concepts;
mod stupid_ai;

use adv_civ::player::Player;
use adv_civ::{GameActivity, GameState};
use bevy::app::App;
use bevy::core::Name;
use bevy::prelude::{AppExtStates, Bundle, Entity, Transform};
use bevy::state::app::StatesPlugin;
use adv_civ::civilization::enums::prelude::GameFaction;
use adv_civ::civilization::components::prelude::*;
use adv_civ::civilization::concepts::census::components::Census;

#[test]
fn it_works() {
    assert!(true);
}

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
                    Token::new(player),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                )).id()
    })
        .collect::<Vec<Entity>>();

    let city_tokens = (0..9).map(|_| {
        app.world_mut()
            .spawn(
                (
                    Name::new("City 1"),
                    CityToken::new(player),
                    Transform::from_xyz(0.0, 0.0, 0.0)
                )
            ).id()
    }
    )
        .collect::<Vec<Entity>>();

    app
        .world_mut()
        .entity_mut(player)
        .insert(
            (
                TokenStock::new(
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
pub fn setup_bevy_app(app_builder: fn(App) -> App) -> App {
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
pub fn create_area(app: &mut App, name: impl Into<String>, id: i32) -> Entity {
    let area = app.world_mut().spawn(
        (
            Name::new(name.into()),
            GameArea::new(id),
            LandPassage::default(),
        )
    ).id();
    area
}

pub fn create_area_w_components<T: Bundle>(app: &mut App, name: &str, components: Option<T>) -> Entity {
    let area = app.world_mut().spawn(
        (
            Name::new(name.to_string()),
            GameArea::new(1),
            LandPassage::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Population::new(3)
        )
    ).id();
    if let Some(components) = components {
        app.world_mut().entity_mut(area).insert(components);
    }
    area
}