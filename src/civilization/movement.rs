use bevy::app::App;
use crate::civilization::civ::{GameActivity, GameActivityStarted};
use bevy::prelude::{Event, EventReader, EventWriter, Plugin, Reflect};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PrepareNextMoverCommand>()
        ;
    }
}

#[derive(Event, Debug, Reflect)]
pub struct PrepareNextMoverCommand {}

pub fn start_movement_activity(
    mut start_activity: EventReader<GameActivityStarted>,
    mut next_mover_command: EventWriter<PrepareNextMoverCommand>
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Movement {
            next_mover_command.send(PrepareNextMoverCommand {});
        }
    }
}

