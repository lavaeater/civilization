use crate::civilization::concepts::city_construction::city_construction_events::*;
use crate::civilization::concepts::city_construction::city_construction_systems::*;
use crate::civilization::concepts::city_construction::city_construction_ui_components::CityConstructionSelectionState;
use crate::civilization::concepts::city_construction::city_construction_ui_systems::*;
use crate::GameActivity;
use bevy::app::{App, Update};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, OnExit, Plugin};

pub struct CityConstructionPlugin;

impl Plugin for CityConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EndPlayerCityConstruction>()
            .add_message::<BuildCityCommand>()
            .init_resource::<CityConstructionSelectionState>()
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
                    // Human player city construction UI systems
                    setup_human_city_construction_options.run_if(in_state(GameActivity::CityConstruction)),
                    spawn_city_construction_controls_ui.run_if(in_state(GameActivity::CityConstruction)),
                    draw_city_construction_highlight.run_if(in_state(GameActivity::CityConstruction)),
                    handle_city_construction_button_clicks.run_if(in_state(GameActivity::CityConstruction)),
                    update_build_site_display.run_if(in_state(GameActivity::CityConstruction)),
                    cleanup_city_construction_ui.run_if(in_state(GameActivity::CityConstruction)),
                ),
            )
            .add_systems(OnExit(GameActivity::CityConstruction), cleanup_city_construction_ui_on_exit);
    }
}
