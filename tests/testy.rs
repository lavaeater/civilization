use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Entity, Name};
use bevy::state::app::StatesPlugin;
use bevy_game::civilization::game_phases::game_activity::*;
use bevy_game::civilization::general::components::*;
use bevy_game::civilization::general::events::*;
use bevy_game::civilization::remove_surplus::systems::*;
use bevy_game::player::Player;
use bevy_game::GameState;

fn setup_player(mut app: App) -> (App, Entity, Vec<Entity>) {
    let player = app.world_mut()
        .spawn(
            (
                Player {},
                Name::new("PLAYER"),
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

    let player: Entity;
    let mut tokens: Vec<Entity>;
    (app, player, tokens) = setup_player(app);

    let mut population = Population::new(4);
    
    population.player_tokens.insert(player, tokens.drain(0..7).collect());
    population.total_population = 7;
    
    app.world_mut().spawn(
        (
            Name::new("egypt"),
            GameArea {},
            LandPassage::default(),
            population
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