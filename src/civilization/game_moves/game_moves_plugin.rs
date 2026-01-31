use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::game_moves::game_moves_systems::*;
use crate::civilization::game_moves::game_moves_triggers::*;
use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoScheduleConfigs, Plugin, Update};
use crate::civilization::concepts::trade::trade_systems::recalculate_trade_moves_for_player;

pub struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AvailableMoves>()
            .add_message::<RecalculatePlayerMoves>()
            .add_systems(
                Update,
                (
                    recalculate_pop_exp_moves_for_player
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    recalculate_movement_moves_for_player.run_if(in_state(GameActivity::Movement)),
                    recalculate_city_construction_moves_for_player
                        .run_if(in_state(GameActivity::CityConstruction)),
                    recalculate_city_support_moves_for_player
                        .run_if(in_state(GameActivity::CheckCitySupport)),
                    recalculate_trade_moves_for_player.run_if(in_state(GameActivity::Trade)),
                ),
            )
            .add_observer(on_add_manual_expansion)
            .add_observer(on_add_perform_movement)
            .add_observer(on_add_is_building)
            .add_observer(on_add_has_just_moved)
            .add_observer(on_add_has_too_many_cities)
            .add_observer(on_add_needs_trade_move);
    }
}
