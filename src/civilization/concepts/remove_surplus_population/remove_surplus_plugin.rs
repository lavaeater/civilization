use crate::GameActivity;
use bevy::prelude::{App, OnEnter, Plugin};
use crate::civilization::remove_surplus_population;

pub struct RemoveSurplusPlugin;

impl Plugin for RemoveSurplusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameActivity::RemoveSurplusPopulation),
            remove_surplus_population,
        );
    }
}
