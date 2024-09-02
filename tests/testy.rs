use bevy::app::Update;
use bevy::prelude::{App, AppExtStates};
use bevy::state::app::StatesPlugin;
use bevy_game::civilization::game_phases::game_activity::GameActivity;
use bevy_game::civilization::general::events::ReturnTokenToStock;
use bevy_game::civilization::remove_surplus::systems::remove_surplus_population;
use bevy_game::GameState;

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


    // // Setup test resource
    // let mut input = ButtonInput::<KeyCode>::default();
    // input.press(KeyCode::Space);


    // Run systems
    app.update();

    // Check resulting changes, one entity has been spawned with `Enemy` component
    // assert_eq!(app.world_mut().query::<&Enemy>().iter(app.world()).len(), 1);
}