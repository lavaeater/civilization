use bevy::prelude::{App, OnEnter, Plugin};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::remove_surplus::systems;

pub struct RemoveSurplusPlugin;

impl Plugin for RemoveSurplusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameActivity::RemoveSurplusPopulation), systems::remove_surplus_population)
        ;
    }
}

