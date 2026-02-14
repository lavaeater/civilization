use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update};
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::concepts::movement::movement_systems::*;
use crate::civilization::concepts::movement::movement_ui_components::MovementSelectionState;
use crate::civilization::concepts::movement::movement_ui_systems::{
    cleanup_movement_ui, cleanup_movement_ui_on_exit, draw_movement_arrows,
    handle_movement_target_click, on_confirm_movement_button, on_end_movement_button,
    pan_camera_to_current_source, setup_human_movement_options, spawn_movement_controls_ui,
    update_source_area_display, update_token_count_display,
};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveTokenFromAreaToAreaCommand>()
            .add_message::<PlayerMovementEnded>()
            .add_message::<NextPlayerStarted>()
            .init_resource::<MovementSelectionState>()
            .add_observer(on_confirm_movement_button)
            .add_observer(on_end_movement_button)
            .add_systems(OnEnter(GameActivity::Movement), start_movement_activity)
            .add_systems(
                Update,
                (
                    prepare_next_mover.run_if(in_state(GameActivity::Movement)),
                    player_end_movement.run_if(in_state(GameActivity::Movement)),
                    move_tokens_from_area_to_area.run_if(in_state(GameActivity::Movement)),
                    animate_token_movement.run_if(in_state(GameActivity::Movement)),
                    // Human player movement UI systems
                    setup_human_movement_options.run_if(in_state(GameActivity::Movement)),
                    spawn_movement_controls_ui.run_if(in_state(GameActivity::Movement)),
                    draw_movement_arrows.run_if(in_state(GameActivity::Movement)),
                    handle_movement_target_click.run_if(in_state(GameActivity::Movement)),
                    update_token_count_display.run_if(in_state(GameActivity::Movement)),
                    update_source_area_display.run_if(in_state(GameActivity::Movement)),
                    pan_camera_to_current_source.run_if(in_state(GameActivity::Movement)),
                    cleanup_movement_ui.run_if(in_state(GameActivity::Movement)),
                ),
            )
            .add_systems(OnExit(GameActivity::Movement), (on_exit_movement, cleanup_movement_ui_on_exit));
    }
}
