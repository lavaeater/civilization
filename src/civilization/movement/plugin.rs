use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::movement::events::{EndPlayerMovement, MoveTokenFromAreaToAreaCommand};
use crate::civilization::movement::systems::{calculate_moves, clear_moves, move_token_from_area_to_area, player_end_movement, prepare_next_mover, start_movement_activity};
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MoveTokenFromAreaToAreaCommand>()
            .add_event::<EndPlayerMovement>()
            .add_systems(
                OnEnter(GameActivity::Movement), start_movement_activity,
            )
            .add_systems(Update, (
                prepare_next_mover.run_if(in_state(GameActivity::Movement)),
                calculate_moves.run_if(in_state(GameActivity::Movement)),
                player_end_movement.run_if(in_state(GameActivity::Movement)),
                move_token_from_area_to_area.run_if(in_state(GameActivity::Movement)),
                )
            )
            .add_systems(
                OnExit(GameActivity::Movement), clear_moves,
            )
        ;
    }
}


