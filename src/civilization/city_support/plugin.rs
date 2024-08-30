use bevy::app::{App, Plugin};
use bevy::prelude::OnEnter;
use crate::civilization::game_phases::game_activity::GameActivity;

pub struct CitySupportPlugin;

impl Plugin for CitySupportPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameActivity::CheckCitySupport), check_city_support)
        ;
    }
}

fn check_city_support() {
    println!("Check city support");
}
