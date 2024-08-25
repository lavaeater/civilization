use bevy::app::App;
use crate::civilization::civ::{GameActivity, GameActivityStarted};
use bevy::prelude::{Event, EventReader, Plugin, Reflect};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event()
        ;
    }
}

#[derive(Event, Debug, Reflect)]
pub struct DetermineNextMoverCommand {}

pub fn start_movement_activity(
    mut start_activity: EventReader<GameActivityStarted>,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Movement {

        }
    }
}