use bevy::prelude::{App, IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update, in_state};

use crate::GameState;
use crate::civilization::concepts::map::CameraFocusQueue;
use crate::civilization::concepts::map::camera_focus::process_camera_focus;
use crate::civilization::concepts::map_editor::map_editor_components::MapEditorState;
use crate::civilization::concepts::map_editor::map_editor_systems::*;
use crate::civilization::concepts::map_editor::map_editor_ui_systems::*;

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapEditorState>()
            .init_resource::<CameraFocusQueue>()
            .add_systems(
                OnEnter(GameState::MapEditor),
                (load_map_editor_handle, spawn_map_editor_ui),
            )
            .add_systems(
                Update,
                (
                    spawn_map_editor_areas,
                    handle_map_editor_click,
                    handle_map_editor_camera_controls,
                    process_camera_focus,
                    draw_map_editor_gizmos,
                    save_map_editor,
                    update_map_editor_ui,
                )
                    .run_if(in_state(GameState::MapEditor)),
            )
            .add_systems(
                OnExit(GameState::MapEditor),
                (cleanup_map_editor, cleanup_map_editor_ui),
            );
    }
}
