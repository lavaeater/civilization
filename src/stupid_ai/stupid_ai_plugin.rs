use crate::stupid_ai::stupid_ai_events::{SelectStupidMove, StupidAiEvent};
use crate::stupid_ai::{stupid_ai_systems, stupid_ai_triggers};
use crate::GameState;
use bevy::app::{Plugin, Update};
use bevy::prelude::{in_state, App, IntoSystemConfigs};

pub struct StupidAiPlugin;

impl Plugin for StupidAiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<StupidAiEvent>()
            .add_event::<SelectStupidMove>()
            .add_systems(Update, (
                stupid_ai_systems::setup_stupid_ai.run_if(in_state(GameState::Playing)),
                stupid_ai_systems::select_stupid_move
                    .run_if(
                        in_state(GameState::Playing))),
            )
            .observe(stupid_ai_triggers::on_add_available_moves)
        ;
    }
}

