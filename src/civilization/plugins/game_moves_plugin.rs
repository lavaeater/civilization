use crate::civilization::events::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::systems::game_moves_systems::{recalculate_city_construction_moves_for_player, recalculate_city_support_moves_for_player, recalculate_movement_moves_for_player, recalculate_pop_exp_moves_for_player};
use crate::civilization::triggers::game_moves_triggers::{on_add_has_just_moved, on_add_has_too_many_cities, on_add_is_building, on_add_manual_expansion, on_add_perform_movement};
use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoSystemConfigs, Plugin, Update};
use crate::civilization::components::game_moves_components::AvailableMoves;

pub struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AvailableMoves>()
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, (
                recalculate_pop_exp_moves_for_player.run_if(in_state(GameActivity::PopulationExpansion)),
                recalculate_movement_moves_for_player.run_if(in_state(GameActivity::Movement)),
                recalculate_city_construction_moves_for_player.run_if(in_state(GameActivity::CityConstruction)),
                recalculate_city_support_moves_for_player.run_if(in_state(GameActivity::CheckCitySupport)),
            ))
            .observe(on_add_manual_expansion)
            .observe(on_add_perform_movement)
            .observe(on_add_is_building)
            .observe(on_add_has_just_moved)
            .observe(on_add_has_too_many_cities)
        ;
    }
}

