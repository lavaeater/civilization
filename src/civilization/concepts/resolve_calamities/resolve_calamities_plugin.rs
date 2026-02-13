use crate::GameActivity;
use bevy::prelude::{App, Commands, Entity, Has, Name, NextState, OnEnter, Plugin, Query, ResMut, With};
use crate::civilization::{BuiltCity, GameArea, Population};

pub struct ResolveCalamitiesPlugin;

impl Plugin for ResolveCalamitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameActivity::ResolveCalamities),
            resolve_calamities,
        );
    }
}

pub fn resolve_calamities(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut commands: Commands,
) {
    
    
}
