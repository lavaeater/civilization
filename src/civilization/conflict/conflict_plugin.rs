use bevy::app::App;
use bevy::prelude::{IntoSystemConfigs, OnEnter, OnExit, Plugin, Update};
use crate::civilization::conflict::conflict_systems::*;
use crate::GameActivity;

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), find_conflict_zones)
            .observe(on_add_unresolved_conflict)
            .observe(on_add_unresolved_city_conflict)
        ;
    }
}

