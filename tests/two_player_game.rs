mod common;

use bevy::prelude::{NextState, Update};
use bevy::prelude::NextState::Pending;
use bevy_game::civilization::city_support::city_support_events::CheckPlayerCitySupport;
use bevy_game::civilization::city_support::city_support_systems::check_city_support;
use bevy_game::civilization::general::general_enums::GameActivity;
use common::{setup_bevy_app, setup_player};
use crate::common::create_area;

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
            .add_systems(Update, check_city_support)
        ;
        app
    });

    setup_player(&mut app, "Player 1");
    create_area(&mut app, "Egypt");

    app.update();

    let state = app.world().get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, Pending(GameActivity::PopulationExpansion)));
}