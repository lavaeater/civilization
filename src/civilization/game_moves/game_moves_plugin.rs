use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoSystemConfigs, Plugin, Update};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::game_moves::{game_moves_systems, game_moves_triggers};

pub struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(
                Update, (
                    game_moves_systems::recalculate_pop_exp_moves_for_player.run_if(in_state(GameActivity::PopulationExpansion)),
                ),
            )
            .observe(game_moves_triggers::on_add_manual_expansion)
        ;
    }
}

