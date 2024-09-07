use bevy::app::Plugin;
use bevy::prelude::{App, Component, OnEnter, Reflect};
use crate::GameState;

pub struct StupidAiPlugin;

impl Plugin for StupidAiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), ())
        ;
    }

}

#[derive(Component, Debug, Reflect)]
pub struct StupidAi;

