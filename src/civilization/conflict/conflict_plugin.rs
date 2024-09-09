use bevy::app::App;
use bevy::prelude::{OnEnter, OnExit, Plugin};
use crate::civilization::conflict::conflict_systems::*;
use crate::GameActivity;

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), find_conflict_zones)
            .add_systems(
                OnExit(GameActivity::Conflict), resolve_conflicts)
        ;
    }
}

