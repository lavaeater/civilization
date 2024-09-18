use bevy::app::{App, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin};
use crate::civilization::city_construction::city_construction_events::{BuildCity, EndPlayerCityConstruction};
use crate::civilization::city_construction::city_construction_systems::{build_city, city_building_gate, end_player_city_construction, on_enter_city_construction};
use crate::GameActivity;

pub struct CityConstructionPlugin;

impl Plugin for CityConstructionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EndPlayerCityConstruction>()
            .add_event::<BuildCity>()
            .add_systems(OnEnter(GameActivity::CityConstruction), on_enter_city_construction)
            .add_systems(Update, (
                end_player_city_construction.run_if(in_state(GameActivity::CityConstruction)),
                build_city.run_if(in_state(GameActivity::CityConstruction)),
                city_building_gate.run_if(in_state(GameActivity::CityConstruction)),
            ))
        ;
    }
}
