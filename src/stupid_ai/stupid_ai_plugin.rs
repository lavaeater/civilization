use crate::stupid_ai::prelude::*;
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
                setup_stupid_ai.run_if(in_state(GameState::Playing)),
                select_stupid_move
                    .run_if(
                        in_state(GameState::Playing))),
            )
            .add_observer(on_add_available_moves)
        ;
    }
}

