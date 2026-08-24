#![allow(clippy::type_complexity)]

pub mod agent_api;
pub mod audio;
pub mod civilization;
pub mod loading;
pub mod menu;
pub mod network_client;
pub mod player;
pub mod stupid_ai;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use crate::civilization::sandbox_plugin::SandboxPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use civilization::CivilizationPlugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Sandbox,
    Menu,
    /// Standalone map area/connection editor, reachable from the main menu.
    MapEditor,
    /// Connected to a multiplayer server: the local rules engine stays
    /// dormant; `network_client` drives everything from protocol messages.
    Online,
}

/// Resource that indicates the game is paused. When present, game systems
/// should be suspended but `GameState::Playing` remains active so that
/// `GameActivity` sub-state is preserved.
#[derive(Resource, Default)]
pub struct GamePaused;

#[derive(
    SubStates,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Default,
    Reflect,
    serde::Serialize,
    serde::Deserialize,
)]
#[source(GameState = GameState::Playing)]
pub enum GameActivity {
    #[default]
    PrepareGame,
    StartGame,
    CollectTaxes,
    PopulationExpansion,
    Census,
    ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupportAfterRemoveSurplusPopulation,
    AcquireTradeCards,
    Trade,
    ResolveCalamities,
    CheckCitySupportAfterResolveCalamities,
    AcquireCivilizationCards,
    /// Rule 31.71: after civ-card purchases, players discard commodity cards
    /// (of their choice) down to the 8-card retention limit.
    ShedCommodityCards,
    MoveSuccessionMarkers,
    /// Terminal state entered when a player reaches a finish square (rule 34.1A).
    /// Final scoring (rule 35) runs on entry; no phase transitions out of here.
    GameOver,
}

impl From<&GameActivity> for adv_civ_protocol::NetPhase {
    fn from(activity: &GameActivity) -> Self {
        use adv_civ_protocol::NetPhase;
        match activity {
            GameActivity::PrepareGame => NetPhase::PrepareGame,
            GameActivity::StartGame => NetPhase::StartGame,
            GameActivity::CollectTaxes => NetPhase::CollectTaxes,
            GameActivity::PopulationExpansion => NetPhase::PopulationExpansion,
            GameActivity::Census => NetPhase::Census,
            GameActivity::ShipConstruction => NetPhase::ShipConstruction,
            GameActivity::Movement => NetPhase::Movement,
            GameActivity::Conflict => NetPhase::Conflict,
            GameActivity::CityConstruction => NetPhase::CityConstruction,
            GameActivity::RemoveSurplusPopulation => NetPhase::RemoveSurplusPopulation,
            GameActivity::CheckCitySupportAfterRemoveSurplusPopulation => {
                NetPhase::CheckCitySupportAfterRemoveSurplusPopulation
            }
            GameActivity::AcquireTradeCards => NetPhase::AcquireTradeCards,
            GameActivity::Trade => NetPhase::Trade,
            GameActivity::ResolveCalamities => NetPhase::ResolveCalamities,
            GameActivity::CheckCitySupportAfterResolveCalamities => {
                NetPhase::CheckCitySupportAfterResolveCalamities
            }
            GameActivity::AcquireCivilizationCards => NetPhase::AcquireCivilizationCards,
            GameActivity::ShedCommodityCards => NetPhase::ShedCommodityCards,
            GameActivity::MoveSuccessionMarkers => NetPhase::MoveSuccessionMarkers,
            GameActivity::GameOver => NetPhase::GameOver,
        }
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            SandboxPlugin,
            crate::civilization::MapEditorPlugin,
            CivilizationPlugin,
            crate::network_client::NetworkClientPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((LogDiagnosticsPlugin::default(),));
        }
    }
}
