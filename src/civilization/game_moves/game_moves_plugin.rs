use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::game_moves::game_moves_systems::{recalculate_movement_moves_for_player, recalculate_pop_exp_moves_for_player};
use crate::civilization::game_moves::game_moves_triggers::{on_add_manual_expansion, on_add_perform_movement};
use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoSystemConfigs, Plugin, Update};

pub struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(
                Update, (
                    recalculate_pop_exp_moves_for_player.run_if(in_state(GameActivity::PopulationExpansion)),
                    recalculate_movement_moves_for_player.run_if(in_state(GameActivity::Movement)),                ),
            )
            .observe(on_add_manual_expansion)
            .observe(on_add_perform_movement)
        ;
    }
}

