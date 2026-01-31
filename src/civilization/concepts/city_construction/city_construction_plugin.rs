use crate::civilization::concepts::city_construction::city_construction_events::*;
use crate::civilization::concepts::city_construction::city_construction_systems::*;
use crate::GameActivity;
use bevy::app::{App, Update};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Plugin};

pub struct CityConstructionPlugin;

impl Plugin for CityConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EndPlayerCityConstruction>()
            .add_message::<BuildCityCommand>()
            .add_systems(
                OnEnter(GameActivity::CityConstruction),
                on_enter_city_construction,
            )
            .add_systems(
                Update,
                (
                    end_player_city_construction.run_if(in_state(GameActivity::CityConstruction)),
                    build_city.run_if(in_state(GameActivity::CityConstruction)),
                    city_building_gate.run_if(in_state(GameActivity::CityConstruction)),
                ),
            );
    }
}
