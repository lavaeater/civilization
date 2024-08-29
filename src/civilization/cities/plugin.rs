use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};
use crate::civilization::cities::systems;
use crate::civilization::game_phases::game_activity::GameActivity;
pub struct CityPlugin;

impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameActivity::CityConstruction), systems::setup_players_and_cities)
        ;
    }
}
