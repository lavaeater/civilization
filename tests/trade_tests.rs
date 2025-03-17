mod common;

/**
Identify some nifty test cases for us here. 
 */
#[test]
fn start_game() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, start_check_city_support)
        ;
        app
    });

    setup_player(&mut app, "Player 1", GameFaction::Egypt);
    create_area(&mut app, "Egypt", 1);

    app.update();

    let state = app.world().get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, Pending(GameActivity::AcquireTradeCards)));
}
