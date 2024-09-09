use crate::civilization::movement::movement_events::{PlayerMovementEnded, MoveTokenFromAreaToAreaCommand, NextPlayerStarted, InitAllAreas, ClearAllMoves};
use crate::civilization::movement::movement_systems::{calculate_moves, clear_all_moves, clear_moves, init_all_areas, move_tokens_from_area_to_area, player_end_movement, prepare_next_mover, start_movement_activity};
use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update};
use crate::GameActivity;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MoveTokenFromAreaToAreaCommand>()
            .add_event::<PlayerMovementEnded>()
            .add_event::<NextPlayerStarted>()
            .add_event::<InitAllAreas>()
            .add_event::<ClearAllMoves>()
            .add_systems(
                OnEnter(GameActivity::Movement), start_movement_activity,
            )
            .add_systems(Update,
                         (
                             clear_all_moves.run_if(in_state(GameActivity::Movement)),
                             prepare_next_mover.run_if(in_state(GameActivity::Movement)),
                             init_all_areas.run_if(in_state(GameActivity::Movement)),
                             calculate_moves.run_if(in_state(GameActivity::Movement)),
                             player_end_movement.run_if(in_state(GameActivity::Movement)),
                             move_tokens_from_area_to_area.run_if(in_state(GameActivity::Movement)),
                         ),
            )
            .add_systems(
                OnExit(GameActivity::Movement), clear_moves,
            )
        ;
    }
}

