use crate::stupid_ai::prelude::*;
use crate::{GameActivity, GameState};
use bevy::app::{Plugin, Update};
use bevy::prelude::{App, IntoScheduleConfigs, in_state};

pub struct StupidAiPlugin;

impl Plugin for StupidAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StupidAiMessage>()
            .add_message::<SelectStupidMove>()
            .add_systems(
                Update,
                (
                    setup_stupid_ai.run_if(in_state(GameState::Playing)),
                    select_stupid_pop_exp.run_if(in_state(GameActivity::PopulationExpansion)),
                    select_stupid_movement.run_if(in_state(GameActivity::Movement)),
                    select_stupid_city_building.run_if(in_state(GameActivity::CityConstruction)),
                    select_stupid_city_elimination.run_if(in_state(GameActivity::CheckCitySupport)),
                    select_stupid_trade_move.run_if(in_state(GameActivity::Trade)),
                ),
            )
            .add_observer(on_add_available_moves);
    }
}
