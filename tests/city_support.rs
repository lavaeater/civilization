mod common;

use bevy::prelude::{NextState, Update};
use bevy::prelude::NextState::Pending;
use bevy_game::civilization::city_support::plugin::{check_city_support, CheckPlayerCitySupport};
use bevy_game::civilization::game_phases::game_activity::GameActivity;
use common::{setup_player, setup_bevy_app};
use crate::common::create_area;

#[test]
fn given_a_game_with_no_cities_the_state_is_moved_forward() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, check_city_support)
        ;
        app
    });
    
    setup_player(&mut  app, "Player 1");
    create_area(&mut app, "Egypt");
    
    app.update();

    let state = app.world().get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, Pending(GameActivity::PopulationExpansion)));
}