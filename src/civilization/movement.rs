use bevy::app::{App, Update};
use crate::civilization::civ::{GameActivity, GameActivityStarted};
use bevy::prelude::{in_state, Event, EventReader, EventWriter, IntoSystemConfigs, Plugin, Reflect};
use crate::GameState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PrepareNextMoverCommand>()
            .add_systems(
                Update, (
                    start_movement_activity
                        .run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug, Reflect)]
pub struct PrepareNextMoverCommand {}

pub fn start_movement_activity(
    mut start_activity: EventReader<GameActivityStarted>,
    mut next_mover_command: EventWriter<PrepareNextMoverCommand>,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Movement {
            next_mover_command.send(PrepareNextMoverCommand {});
        }
    }
}

