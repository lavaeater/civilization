use crate::{create_area, setup_bevy_app, setup_player};
use adv_civ::GameActivity;
use bevy::prelude::NextState::Pending;
use bevy::prelude::{NextState, Update};
use adv_civ::civilization::concepts::check_city_support::check_city_support_events::CheckPlayerCitySupport;
use adv_civ::civilization::concepts::check_city_support::check_city_support_systems::start_check_city_support;
use adv_civ::civilization::enums::GameFaction;
/***
We are going to write a test that actually plays the game for us with two players. It is going
to be sooo much work, but perhaps it will be worth it?

It is either this or making some kind of scripting for the commands. I will do these in parallell...
 */

#[test]
fn start_game() {
    let mut app = setup_bevy_app(|mut app| {
        app.add_message::<CheckPlayerCitySupport>()
            .add_systems(Update, start_check_city_support);
        app
    });

    setup_player(&mut app, "Player 1", GameFaction::Egypt);
    create_area(&mut app, "Egypt", 1);

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::AcquireTradeCards)));
}
