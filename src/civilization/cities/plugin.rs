use crate::civilization::cities::events::EndCityConstructionActivity;
use crate::civilization::cities::systems;
use crate::civilization::cities::systems::end_city_construction_activity;
use crate::civilization::game_phases::game_activity::GameActivity;
use bevy::app::{App, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin};

pub struct CityPlugin;

impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EndCityConstructionActivity>()
            .add_systems(OnEnter(GameActivity::CityConstruction), systems::setup_players_and_cities)
            .add_systems(Update, end_city_construction_activity.run_if(in_state(GameActivity::CityConstruction)))
        ;
    }
}
