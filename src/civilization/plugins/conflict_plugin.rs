use crate::civilization::systems::conflict_systems::*;
use crate::civilization::triggers::conflict_triggers::{on_add_unresolved_city_conflict, on_add_unresolved_conflict};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin, Update};

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), find_conflict_zones)
            .add_systems(Update, (conflict_gate).run_if(in_state(GameActivity::Conflict)))
            .observe(on_add_unresolved_conflict)
            .observe(on_add_unresolved_city_conflict)
        ;
    }
}

