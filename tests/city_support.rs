mod common;

use bevy::prelude::{NextState, Update};
use bevy::prelude::NextState::Pending;
use bevy_game::civilization::city_support::plugin::check_city_support;
use bevy_game::civilization::game_phases::game_activity::GameActivity;
use common::{setup_player, setup_bevy_app};
use crate::common::create_area;

#[test]
fn given_a_game_with_no_cities_the_state_is_moved_forward() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_systems(Update, check_city_support)
        ;
        app
    });
    
    let (player, tokens) = setup_player(&mut  app, "Player 1");
    let area = create_area(&mut app, "Egypt");
    
    app.update();

    let a = Pending(GameActivity::PopulationExpansion);
    let state = app.world().resource_ref::<GameActivity>>().into_inner();
    
    
    
    if *state == a {
        println!("State is {:?}", state);
    }
    assert!(state == a);
}