use crate::GameActivity;
use bevy::app::App;
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update};
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::concepts::movement::movement_systems::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveTokenFromAreaToAreaCommand>()
            .add_message::<PlayerMovementEnded>()
            .add_message::<NextPlayerStarted>()
            .add_systems(OnEnter(GameActivity::Movement), start_movement_activity)
            .add_systems(
                Update,
                (
                    prepare_next_mover.run_if(in_state(GameActivity::Movement)),
                    player_end_movement.run_if(in_state(GameActivity::Movement)),
                    move_tokens_from_area_to_area.run_if(in_state(GameActivity::Movement)),
                ),
            )
            .add_systems(OnExit(GameActivity::Movement), on_exit_movement);
    }
}
