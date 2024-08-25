use std::collections::VecDeque;
use bevy::app::{App, Update};
use crate::civilization::civ::{GameActivity, GameActivityEnded, GameActivityStarted, Population};
use bevy::prelude::{in_state, Children, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Plugin, Query, Reflect, Res, ResMut, With};
use crate::civilization::census::GameInfoAndStuff;
use crate::GameState;
use VecDeque::pop_front;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PrepareNextMoverCommand>()
            .add_systems(
                Update, (
                    start_movement_activity
                        .run_if(in_state(GameState::Playing)),
                    prepare_next_mover
                        .run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug, Reflect)]
pub struct PrepareNextMoverCommand {}

#[derive(Component, Debug, Reflect)]
pub struct PerformingMovement;

pub fn start_movement_activity(
    mut start_activity: EventReader<GameActivityStarted>,
    mut next_mover_command: EventWriter<PrepareNextMoverCommand>,
    mut game_info: ResMut<GameInfoAndStuff>,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Movement {
            game_info.left_to_move = game_info.census_order.clone();
            game_info.left_to_move.reverse();
            next_mover_command.send(PrepareNextMoverCommand {});
        }
    }
}

pub fn prepare_next_mover(
    mut next_mover: EventReader<PrepareNextMoverCommand>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut move_over: EventWriter<GameActivityEnded>,
    moveable_tokens: Query<&Children, With<Population>>,
    mut commands: Commands
) {
    for _ in next_mover.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            commands.entity(to_move).insert((PerformingMovement{}));
            game_info.current_mover = Some(to_move);
        } else {
            // We're done bro
            game_info.current_mover = None;
            move_over.send(GameActivityEnded(GameActivity::Movement));
        }
    }
}