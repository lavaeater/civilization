use crate::civilization::concepts::succession::succession_systems::advance_succession_markers;
use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};

pub struct SuccessionPlugin;

impl Plugin for SuccessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameActivity::MoveSuccessionMarkers),
            advance_succession_markers,
        );
    }
}
