#![allow(clippy::type_complexity)]

pub mod actions;
pub mod audio;
pub mod loading;
pub mod menu;
pub mod player;
pub mod civilization;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use crate::civilization::general::plugin::CivilizationPlugin;
use bevy::app::App;
// #[cfg(debug_assertions)]
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                CivilizationPlugin,
            ));

        // #[cfg(debug_assertions)]
        // {
        //     app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        // }
    }
}
