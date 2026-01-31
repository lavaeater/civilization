use crate::GameActivity;
use bevy::prelude::{App, OnEnter, Plugin};
use crate::civilization::concepts::remove_surplus_population::remove_surplus_systems;

pub struct RemoveSurplusPlugin;

impl Plugin for RemoveSurplusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameActivity::RemoveSurplusPopulation),
            remove_surplus_systems::remove_surplus_population,
        );
    }
}
