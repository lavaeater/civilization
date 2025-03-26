use bevy::app::{App, Plugin};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Update};
use crate::civilization::events::city_support_events::{CheckCitySupportStatus, CheckPlayerCitySupport, EliminateCity};
use crate::civilization::systems::city_support_systems;
use crate::GameActivity;

pub struct CitySupportPlugin;

impl Plugin for CitySupportPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EliminateCity>()
            .add_event::<CheckPlayerCitySupport>()
            .add_event::<CheckCitySupportStatus>()
            .add_systems(OnEnter(GameActivity::CheckCitySupport), city_support_systems::start_check_city_support)
            .add_systems(Update,
                         (
                             city_support_systems::eliminate_city.run_if(in_state(GameActivity::CheckCitySupport)),
                             city_support_systems::check_player_city_support.run_if(in_state(GameActivity::CheckCitySupport)),
                             city_support_systems::check_status.run_if(in_state(GameActivity::CheckCitySupport)),
                         ),
            )
        ;
    }
}

