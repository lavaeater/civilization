#![allow(clippy::type_complexity)]

pub mod actions;
pub mod audio;
pub mod loading;
pub mod menu;
pub mod player;
pub mod civilization;
pub mod stupid_ai;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

use crate::actions::ActionsPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use crate::civilization::sandbox_plugin::SandboxPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use civilization::plugins::CivilizationPlugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Sandbox,
    Menu,
}

/// Resource that indicates the game is paused. When present, game systems
/// should be suspended but `GameState::Playing` remains active so that
/// `GameActivity` sub-state is preserved.
#[derive(Resource, Default)]
pub struct GamePaused;

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect, serde::Serialize, serde::Deserialize)]
#[source(GameState = GameState::Playing)]
pub enum GameActivity {
    #[default]
    PrepareGame,
    StartGame,
    // CollectTaxes,
    PopulationExpansion,
    Census,
    // ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupportAfterRemoveSurplusPopulation,
    AcquireTradeCards,
    Trade,
    ResolveCalamities,
    CheckCitySupportAfterResolveCalamities,
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
            SandboxPlugin,
            CivilizationPlugin
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                LogDiagnosticsPlugin::default(),
            ));
        }
    }
}
