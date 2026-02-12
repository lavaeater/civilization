use crate::civilization::concepts::conflict::conflict_systems::{find_conflict_zones};
use crate::civilization::concepts::conflict::conflict_triggers::{
    on_add_unresolved_city_conflict, on_add_unresolved_conflict,
};
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Plugin, Resource, Update};

pub struct ConflictPlugin;

#[derive(Resource, Default)]
pub struct ConflictCounterResource(pub usize);

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ConflictCounterResource>()
            .add_systems(OnEnter(GameActivity::Conflict), find_conflict_zones)
            .add_observer(on_add_unresolved_conflict)
            .add_observer(on_add_unresolved_city_conflict);
    }
}
