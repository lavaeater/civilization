use crate::civilization::components::*;
use crate::civilization::concepts::*;
use crate::civilization::enums::GameFaction;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::game_moves::GameMovesPlugin;
use crate::civilization::general_systems::{connect_areas, fix_token_positions, move_tokens_from_stock_to_area, print_names_of_phases, start_game};
use crate::civilization::triggers::on_add_return_token_to_stock;
use crate::stupid_ai::*;
use crate::{GameActivity, GameState};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, AppExtStates, IntoScheduleConfigs, OnEnter, Resource};

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        // Use DebugOptions::test_manual_pop_exp() to test manual population expansion
        app.insert_resource(DebugOptions::default())
        .register_type::<Token>()
        .register_type::<LandPassage>()
        .register_type::<TokenStock>()
        .register_type::<GameArea>()
        .register_type::<Population>()
        .register_type::<PlayerAreas>()
        .register_type::<Faction>()
        .register_type::<Treasury>()
        .add_message::<MoveTokensFromStockToAreaCommand>()
        .add_sub_state::<GameActivity>()
        .add_systems(
            Update,
            (print_names_of_phases.run_if(in_state(GameState::Playing)),),
        )
        .add_plugins((
            // CommandsPlugin,
            PopulationExpansionPlugin,
            CensusPlugin,
            MovementPlugin,
            ConflictPlugin,
            TradePlugin,
        ))
        .add_plugins((
            CityConstructionPlugin,
            RemoveSurplusPlugin,
            CitySupportPlugin,
            StupidAiPlugin,
            GameMovesPlugin,
            TradeCardPlugin,
            MapPlugin,
            TradeUiPlugin,
        ))
        .add_systems(OnEnter(GameActivity::StartGame), start_game)
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(GameInfoAndStuff::default())
        .add_systems(
            Update,
            (
                connect_areas.run_if(in_state(GameState::Playing)),
                move_tokens_from_stock_to_area.run_if(in_state(GameState::Playing)),
                fix_token_positions.run_if(in_state(GameState::Playing)),
            ),
        )
        .add_observer(on_add_return_token_to_stock);
    }
}

#[derive(Resource)]
pub struct DebugOptions {
    pub add_human_player: bool,
    pub human_faction: GameFaction,
    pub human_always_pulls_trade_cards: bool,
    pub ai_always_pulls_trade_cards: bool,
    pub human_starts_with_trade_cards: bool,
    pub auto_trading: bool,
    pub print_selected_moves: bool,
    pub number_of_players: usize,
    /// If set, the game will start at this activity instead of the normal flow.
    pub start_at_activity: Option<GameActivity>,
    /// Number of tokens to give the human player in stock (for testing limited expansion).
    /// If None, uses the default 47 tokens.
    pub human_token_count: Option<usize>,
    /// Number of areas to populate for the human player at start (for testing expansion).
    /// If None, uses normal start (1 token in start area).
    pub human_starting_areas: Option<usize>,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            add_human_player: true,
            human_faction: GameFaction::Assyria,
            human_always_pulls_trade_cards: true,
            ai_always_pulls_trade_cards: false,
            human_starts_with_trade_cards: false,
            auto_trading: false,
            print_selected_moves: false,
            number_of_players: 7,
            start_at_activity: None,
            human_token_count: None,
            human_starting_areas: None,
        }
    }
}

impl DebugOptions {
    /// Create a debug configuration for testing manual population expansion.
    /// This gives the human player limited tokens and multiple populated areas.
    pub fn test_manual_pop_exp() -> Self {
        Self {
            add_human_player: true,
            human_faction: GameFaction::Assyria,
            human_always_pulls_trade_cards: false,
            ai_always_pulls_trade_cards: false,
            human_starts_with_trade_cards: false,
            auto_trading: false,
            print_selected_moves: true,
            number_of_players: 2,
            start_at_activity: None,
            // Give human only 2 tokens so they can't auto-expand all areas
            human_token_count: Some(2),
            // Populate 3 areas so manual choice is required
            human_starting_areas: Some(3),
        }
    }
}
