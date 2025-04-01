#![allow(clippy::type_complexity)]
// #![feature(impl_trait_in_assoc_type, associated_type_defaults)]

pub mod actions;
pub mod audio;
pub mod loading;
pub mod menu;
pub mod player;
pub mod civilization;
pub mod stupid_ai;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use bevy::app::App;
// #[cfg(debug_assertions)]
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use civilization::plugins::civilization_plugin::CivilizationPlugin;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect)]
#[source(GameState = GameState::Playing)]
pub enum GameActivity {
    #[default]
    StartGame,
    // CollectTaxes,
    PopulationExpansion,
    Census,
    // ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupport,
    AcquireTradeCards,
    Trade,
    // ResolveCalamities,
    // AcquireCivilizationCards,
    // MoveSuccessionMarkers,
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
