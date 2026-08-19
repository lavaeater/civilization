use crate::civilization::concepts::succession::ast_ui_systems::{
    spawn_ast_ui, toggle_ast_ui, update_ast_markers,
};
use crate::civilization::concepts::succession::succession_components::{AstTrack, RoundLimit};
use crate::civilization::concepts::succession::succession_systems::{
    advance_succession_markers, determine_winner,
};
use crate::{GameActivity, GameState};
use bevy::app::{App, Update};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Plugin};

pub struct SuccessionPlugin;

impl Plugin for SuccessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AstTrack>()
            .init_resource::<RoundLimit>()
            .add_systems(
                OnEnter(GameActivity::MoveSuccessionMarkers),
                advance_succession_markers,
            )
            .add_systems(OnEnter(GameActivity::GameOver), determine_winner)
            .add_systems(OnEnter(GameActivity::StartGame), spawn_ast_ui)
            .add_systems(
                Update,
                (update_ast_markers, toggle_ast_ui).run_if(in_state(GameState::Playing)),
            );
    }
}
