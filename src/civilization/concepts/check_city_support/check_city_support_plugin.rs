use crate::civilization::concepts::check_city_support::check_city_support_events::*;
use crate::civilization::concepts::check_city_support::check_city_support_systems::*;
use crate::GameActivity;
use bevy::app::{App, Plugin};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Update};

pub struct CitySupportPlugin;

impl Plugin for CitySupportPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EliminateCity>()
            .add_message::<CheckPlayerCitySupport>()
            .add_message::<CheckCitySupportStatus>()
            .add_systems(
                OnEnter(GameActivity::CheckCitySupport),
                start_check_city_support,
            )
            .add_systems(
                Update,
                (
                    eliminate_city
                        .run_if(in_state(GameActivity::CheckCitySupport)),
                    check_player_city_support
                        .run_if(in_state(GameActivity::CheckCitySupport)),
                    check_status
                        .run_if(in_state(GameActivity::CheckCitySupport)),
                ),
            );
    }
}
