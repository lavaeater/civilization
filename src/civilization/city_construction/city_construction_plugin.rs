use bevy::app::{App, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin};
use crate::civilization::city_construction::city_construction_events::{BuildCity, EndCityConstructionActivity};
use crate::civilization::city_construction::city_construction_systems::{build_city, check_if_done_building, end_city_construction_activity, setup_players_and_cities};
use crate::GameActivity;

pub struct CityConstructionPlugin;

impl Plugin for CityConstructionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EndCityConstructionActivity>()
            .add_event::<BuildCity>()
            .add_systems(OnEnter(GameActivity::CityConstruction), setup_players_and_cities)
            .add_systems(Update, (
                end_city_construction_activity.run_if(in_state(GameActivity::CityConstruction)),
                build_city.run_if(in_state(GameActivity::CityConstruction)),
                check_if_done_building.run_if(in_state(GameActivity::CityConstruction)),
            ))
        ;
    }
}
