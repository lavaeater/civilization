use crate::GameState;
use bevy::app::Update;
use bevy::prelude::{in_state, AppExtStates, IntoSystemConfigs, Plugin};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::game_phases::systems::print_names_of_phases;

pub struct GamePhasesPlugin;

impl Plugin for GamePhasesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_sub_state::<GameActivity>()
            .add_systems(
                Update, (
                    print_names_of_phases.run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

