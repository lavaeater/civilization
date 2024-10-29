use crate::GameActivity;
use bevy::prelude::{OnEnter, OnExit, Plugin};
use crate::civilization::concepts::census::components::Census;
use crate::civilization::concepts::census::systems::{check_areas_for_population, perform_census};

pub struct CensusPlugin;

impl Plugin for CensusPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .register_type::<Census>()
            .add_systems(OnEnter(GameActivity::Census), perform_census)
            .add_systems(OnExit(GameActivity::Census), check_areas_for_population)
        ;
    }
}

