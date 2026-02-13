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
                OnEnter(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation),
                start_check_city_support,
            )
            .add_systems(
                OnEnter(GameActivity::CheckCitySupportAfterResolveCalamities),
                start_check_city_support,
            )
            .add_systems(
                Update,
                (
                    eliminate_city
                        .run_if(in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation))
                        .run_if(in_state(GameActivity::CheckCitySupportAfterResolveCalamities)),
                    check_player_city_support
                        .run_if(in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation))
                        .run_if(in_state(GameActivity::CheckCitySupportAfterResolveCalamities)),
                    check_status_after_remove_surplus_population
                        .run_if(in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation)),
                    check_status_after_resolve_calamities
                        .run_if(in_state(GameActivity::CheckCitySupportAfterResolveCalamities))
                ),
            );
    }
}
