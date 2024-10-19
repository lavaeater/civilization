use bevy::prelude::{App, OnEnter, Plugin};
use crate::civilization::systems::remove_surplus_systems;
use crate::GameActivity;

pub struct RemoveSurplusPlugin;

impl Plugin for RemoveSurplusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameActivity::RemoveSurplusPopulation), remove_surplus_systems::remove_surplus_population)
        ;
    }
}

