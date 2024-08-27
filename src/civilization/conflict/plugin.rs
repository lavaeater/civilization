use crate::civilization::game_phases::game_activity::GameActivity;
use bevy::app::App;
use bevy::prelude::{NextState, OnEnter, Plugin, ResMut};

pub struct ConflictPlugin;

impl Plugin for ConflictPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(GameActivity::Conflict), move_back_to_start)
        ;
    }
}

fn move_back_to_start(
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    next_state.set(GameActivity::PopulationExpansion);
}