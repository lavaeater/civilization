
use bevy::prelude::{NextState, Update};
use bevy::prelude::NextState::Pending;
use adv_civ::civilization::enums::prelude::GameFaction;
use adv_civ::civilization::events::prelude::CheckPlayerCitySupport;
use adv_civ::civilization::systems::prelude::start_check_city_support;
use adv_civ::GameActivity;
use crate::{create_area, setup_bevy_app, setup_player};
/***
We are going to write a test that actually plays the game for us with two players. It is going
to be sooo much work, but perhaps it will be worth it?

It is either this or making some kind of scripting for the commands. I will do these in parallell...
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