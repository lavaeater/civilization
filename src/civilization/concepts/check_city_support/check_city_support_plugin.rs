use crate::GameActivity;
use crate::civilization::concepts::check_city_support::check_city_support_events::*;
use crate::civilization::concepts::check_city_support::check_city_support_systems::*;
use crate::civilization::concepts::check_city_support::check_city_support_ui_components::CitySupportSelectionState;
use crate::civilization::concepts::check_city_support::check_city_support_ui_systems::*;
use bevy::app::{App, Plugin};
use bevy::prelude::{IntoScheduleConfigs, OnEnter, OnExit, SystemCondition, Update, in_state};

pub struct CitySupportPlugin;

impl Plugin for CitySupportPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EliminateCity>()
            .add_message::<CheckPlayerCitySupport>()
            .add_message::<CheckCitySupportStatus>()
            .init_resource::<CitySupportSelectionState>()
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
                    eliminate_city.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    check_player_city_support.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    check_status_after_remove_surplus_population.run_if(in_state(
                        GameActivity::CheckCitySupportAfterRemoveSurplusPopulation,
                    )),
                    check_status_after_resolve_calamities.run_if(in_state(
                        GameActivity::CheckCitySupportAfterResolveCalamities,
                    )),
                    // Human player city support UI systems
                    setup_human_city_support_options.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    spawn_city_support_controls_ui.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    draw_city_support_highlight.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    focus_camera_on_support_site.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    handle_city_support_button_clicks.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    update_city_support_site_display.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                    cleanup_city_support_ui.run_if(
                        in_state(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation).or_else(
                            in_state(GameActivity::CheckCitySupportAfterResolveCalamities),
                        ),
                    ),
                ),
            )
            .add_systems(
                OnExit(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation),
                cleanup_city_support_ui_on_exit,
            )
            .add_systems(
                OnExit(GameActivity::CheckCitySupportAfterResolveCalamities),
                cleanup_city_support_ui_on_exit,
            );
    }
}
