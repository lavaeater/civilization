use crate::civilization::concepts::conflict::conflict_systems::{conflict_gate, find_conflict_zones};
use crate::civilization::concepts::conflict::conflict_triggers::{
    on_add_unresolved_city_conflict, on_add_unresolved_conflict,
};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Plugin, Update};

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameActivity::Conflict), find_conflict_zones)
            .add_systems(
                Update,
                (conflict_gate).run_if(in_state(GameActivity::Conflict)),
            )
            .add_observer(on_add_unresolved_conflict)
            .add_observer(on_add_unresolved_city_conflict);
    }
}
