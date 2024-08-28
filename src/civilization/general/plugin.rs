use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::civilization::census::plugin::CensusPlugin;
use crate::civilization::census::resources::GameInfoAndStuff;
use crate::civilization::conflict::plugin::ConflictPlugin;
use crate::civilization::console::commands::CommandsPlugin;
use crate::civilization::game_phases::plugin::GamePhasesPlugin;
use crate::civilization::general::components::{Area, LandPassage, Population, Stock, Token};
use crate::civilization::general::events::{MoveTokensFromStockToAreaCommand, ReturnTokenToStock};
use crate::civilization::general::systems::{connect_areas, move_tokens_from_stock_to_area, setup_game, setup_players, return_token_to_stock};
use crate::civilization::movement::plugin::MovementPlugin;
use crate::civilization::population_expansion::plugin::PopulationExpansionPlugin;
use crate::GameState;

pub struct CivilizationPlugin;

// MovementPlugin,
// CensusPlugin,


/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Token>()
            .register_type::<LandPassage>()
            .register_type::<Stock>()
            .register_type::<Area>()
            .register_type::<Population>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
            .add_event::<ReturnTokenToStock>()
            .add_plugins(
                (
                    GamePhasesPlugin,
                    CommandsPlugin,
                    PopulationExpansionPlugin,
                    CensusPlugin,
                    MovementPlugin,
                    ConflictPlugin
                )
            )
            .add_systems(OnEnter(GameState::Playing), (setup_game, setup_players))
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

