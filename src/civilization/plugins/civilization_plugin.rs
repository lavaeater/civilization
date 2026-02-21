use crate::civilization::components::*;
use crate::civilization::concepts::*;
use crate::civilization::enums::GameFaction;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::game_moves::GameMovesPlugin;
use crate::civilization::general_systems::{connect_areas, fix_token_positions, move_tokens_from_stock_to_area, print_names_of_phases, start_game};
use crate::civilization::resolve_calamities::resolve_calamities_plugin::ResolveCalamitiesPlugin;
use crate::civilization::triggers::on_add_return_token_to_stock;
use crate::civilization::CivilizationInputPlugin;
use crate::player::Player;
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
        .register_type::<Player>()
        .register_type::<Token>()
        .register_type::<LandPassage>()
        .register_type::<TokenStock>()
        .register_type::<GameArea>()
        .register_type::<Population>()
        .register_type::<PlayerAreas>()
        .register_type::<PlayerCities>()
        .register_type::<Faction>()
        .register_type::<Treasury>()
        .register_type::<CityToken>()
        .register_type::<CityTokenStock>()
        .register_type::<BuiltCity>()
        .register_type::<CitySite>()
        .register_type::<FloodPlain>()
        .register_type::<Volcano>()
        .register_type::<CityFlood>()
        .register_type::<NeedsConnections>()
        .register_type::<StartArea>()
        .register_type::<Census>()
        .register_type::<PlayerTradeCards>()
        .register_type::<StupidAi>()
        .register_type::<IsHuman>()
        .add_message::<MoveTokensFromStockToAreaCommand>()
        .add_sub_state::<GameActivity>()
        .add_systems(
            Update,
            (print_names_of_phases.run_if(in_state(GameState::Playing)),),
        )
        .add_plugins((
            PopulationExpansionPlugin,
            CensusPlugin,
            MovementPlugin,
            ConflictPlugin,
            TradePlugin,
            CivilizationInputPlugin,
            CivCardsPlugin,
        ))
        .add_plugins((
            CityConstructionPlugin,
            RemoveSurplusPlugin,
            ResolveCalamitiesPlugin,
            CitySupportPlugin,
            StupidAiPlugin,
            GameMovesPlugin,
            TradeCardPlugin,
            MapPlugin,
            TradeUiPlugin,
            AreaInfoPlugin,
            SaveGamePlugin,
            lava_ui_builder::LavaUiPlugin,
        ))
        .add_systems(OnEnter(GameActivity::StartGame), start_game)
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
    pub log_selected_moves: bool,
    pub number_of_players: usize,
    /// If set, the game will start at this activity instead of the normal flow.
    pub start_at_activity: Option<GameActivity>,
    /// Number of tokens to give the human player in stock (for testing limited expansion).
    /// If None, uses the default 47 tokens.
    pub human_token_count: Option<usize>,
    /// Number of areas to populate for the human player at start (for testing expansion).
    /// If None, uses normal start (1 token in start area).
    pub human_starting_areas: Option<usize>,
    pub specific_state_name: Option<String>,
    /// Delay in seconds before AI processes each move (0.0 = instant)
    pub ai_move_delay_secs: f32,
    pub show_debug_ui: bool,
    pub human_trade_cards: Option<Vec<(TradeCard, usize)>>,
    pub human_civ_cards: Option<Vec<CivCardName>>,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            add_human_player: true,
            human_faction: GameFaction::Assyria,
            human_always_pulls_trade_cards: false,
            ai_always_pulls_trade_cards: false,
            human_starts_with_trade_cards: false,
            auto_trading: false,
            print_selected_moves: true,
            log_selected_moves: false,
            number_of_players: 7,
            start_at_activity: None,
            human_token_count: None,
            human_starting_areas: None,
            specific_state_name: None,
            ai_move_delay_secs: 0.1,
            show_debug_ui: true,
            human_trade_cards: None,
            human_civ_cards: None,
        }
    }
}

impl DebugOptions {
    /// Create a debug configuration for testing manual population expansion.
    /// This gives the human player limited tokens and multiple populated areas.
    pub fn test_civ_cards() -> Self {
        Self {
            add_human_player: true,
            human_faction: GameFaction::Assyria,
            human_always_pulls_trade_cards: false,
            ai_always_pulls_trade_cards: false,
            human_starts_with_trade_cards: false,
            auto_trading: false,
            print_selected_moves: true,
            log_selected_moves: false,
            number_of_players: 7,
            start_at_activity: Some(GameActivity::AcquireCivilizationCards),
            // Give human only 2 tokens so they can't auto-expand all areas
            human_token_count: None,
            // Populate 3 areas so manual choice is required
            human_starting_areas: None,
            specific_state_name: None,
            ai_move_delay_secs: 0.1,
            show_debug_ui: false,
            human_trade_cards: Some(vec![(TradeCard::Wine, 4), (TradeCard::Salt, 4)]),
            human_civ_cards: Some(vec![CivCardName::ClothMaking, CivCardName::Mathematics]),
        }
    }
}
