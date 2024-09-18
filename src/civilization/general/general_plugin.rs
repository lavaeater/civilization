use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, AppExtStates, IntoSystemConfigs, OnEnter};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::civilization::census::census_plugin::CensusPlugin;
use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::city_construction::city_construction_plugin::CityConstructionPlugin;
use crate::civilization::city_support::city_support_plugin::CitySupportPlugin;
use crate::civilization::conflict::conflict_plugin::ConflictPlugin;
use crate::civilization::console::console_commands::CommandsPlugin;
use crate::civilization::general::general_components::{GameArea, LandPassage, Population, PlayerStock, Token};
use crate::civilization::general::general_events::{MoveTokensFromStockToAreaCommand, ReturnTokenToStock};
use crate::civilization::general::general_systems::{connect_areas, move_tokens_from_stock_to_area, setup_game, setup_players, return_token_to_stock, print_names_of_phases, start_game};
use crate::civilization::movement::movement_plugin::MovementPlugin;
use crate::civilization::population_expansion::population_expansion_plugin::PopulationExpansionPlugin;
use crate::civilization::remove_surplus::remove_surplus_plugin::RemoveSurplusPlugin;
use crate::{GameActivity, GameState};
use crate::civilization::game_moves::game_moves_plugin::GameMovesPlugin;
use crate::stupid_ai::stupid_ai_plugin::StupidAiPlugin;

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Token>()
            .register_type::<LandPassage>()
            .register_type::<PlayerStock>()
            .register_type::<GameArea>()
            .register_type::<Population>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
            .add_event::<ReturnTokenToStock>()
            .add_sub_state::<GameActivity>()
            .add_systems(
                Update, (
                    print_names_of_phases.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_plugins(
                (
                    CommandsPlugin,
                    PopulationExpansionPlugin,
                    CensusPlugin,
                    MovementPlugin,
                    ConflictPlugin,
                    CityConstructionPlugin,
                    RemoveSurplusPlugin,
                    CitySupportPlugin,
                    StupidAiPlugin,
                    GameMovesPlugin
                )
            )
            .add_systems(OnEnter(GameState::Playing), (setup_game, setup_players))
            .add_systems(OnEnter(GameActivity::StartGame), start_game)
            .add_plugins(WorldInspectorPlugin::new())
            .insert_resource(GameInfoAndStuff::default())
            .add_systems(
                Update, (
                    connect_areas
                        .run_if(in_state(GameState::Playing)),
                    move_tokens_from_stock_to_area
                        .run_if(in_state(GameState::Playing)),
                    return_token_to_stock
                        .run_if(in_state(GameState::Playing)),
                ));
    }
}

