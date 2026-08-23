use bevy::prelude::{App, IntoScheduleConfigs, OnExit, Plugin, Update, in_state};

use crate::GameActivity;
use crate::civilization::concepts::choose_start_area::choose_start_area_components::StartAreaSelectionState;
use crate::civilization::concepts::choose_start_area::choose_start_area_systems::*;
use crate::civilization::concepts::choose_start_area::choose_start_area_ui_systems::*;

pub struct ChooseStartAreaPlugin;

impl Plugin for ChooseStartAreaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StartAreaSelectionState>()
            .add_systems(
                Update,
                (
                    apply_start_area_choice,
                    handle_start_area_click,
                    draw_start_area_choice_gizmos,
                    spawn_start_area_banner,
                    cleanup_start_area_banner,
                )
                    .run_if(in_state(GameActivity::StartGame)),
            )
            .add_systems(
                OnExit(GameActivity::StartGame),
                cleanup_start_area_banner_on_exit,
            );
    }
}
