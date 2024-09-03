use bevy::app::Update;
use bevy::asset::AssetContainer;
use bevy::prelude::{App, AppExtStates, Entity, Mut, Name};
use bevy::state::app::StatesPlugin;
use clap::ArgAction::Set;
use bevy_game::civilization::census::components::Census;
use bevy_game::civilization::game_phases::game_activity::*;
use bevy_game::civilization::general::events::*;
use bevy_game::civilization::general::components::*;
use bevy_game::civilization::remove_surplus::systems::*;
use bevy_game::GameState;
use bevy_game::player::Player;

fn setup_player(mut app: App) -> App {
    let player = app.world_mut()
        .spawn(
            (
                Player {},
                Name::new(format!("PLAYER")),
                Census { population: 0 },
                Treasury { tokens: vec![] },
            )
        ).id();

    let tokens = (0..47).map(|_| {
        app.world_mut()
            .spawn(
                (
                    Name::new("Token 1"),
                    Token::new(player))).id()
    }
    )
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
                    tokens),
                CityTokenStock::new(
                    9,
                    city_tokens
                )
            )
        );
    app
}

#[test]
fn a_simple_test() {
    // Setup app
    let mut app = App::new();
    app
        .add_plugins(
            StatesPlugin,
        )
        .add_event::<ReturnTokenToStock>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, remove_surplus_population);

    app = setup_player(app);
    
    app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea {},
            LandPassage::default(),
            Population::new(4)
        )
    );

    // // Setup test resource
    // let mut input = ButtonInput::<KeyCode>::default();
    // input.press(KeyCode::Space);


    // Run systems
    app.update();

    // Check resulting changes, one entity has been spawned with `Enemy` component
    // assert_eq!(app.world_mut().query::<&Enemy>().iter(app.world()).len(), 1);
}