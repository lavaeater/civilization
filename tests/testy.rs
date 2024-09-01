use bevy::app::Update;
use bevy::prelude::App;
use bevy_game::civilization::remove_surplus::systems::remove_surplus_population;

#[test]
fn a_simple_test() {
    // Setup app
    let mut app = App::new();

    // Add our systems
    app.add_systems(Update, remove_surplus_population);

    // // Setup test resource
    // let mut input = ButtonInput::<KeyCode>::default();
    // input.press(KeyCode::Space);
    // app.insert_resource(input);

    // Run systems
    app.update();

    // Check resulting changes, one entity has been spawned with `Enemy` component
    // assert_eq!(app.world_mut().query::<&Enemy>().iter(app.world()).len(), 1);
}