use crate::civilization::game_phases::game_activity::GameActivity;
use bevy::app::App;
use bevy::prelude::{OnEnter, OnExit, Plugin};
use crate::civilization::conflict::systems;
pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), systems::find_conflict_zones)
            .add_systems(
                OnExit(GameActivity::Conflict), systems::resolve_conflicts)
        ;
    }
}

