use crate::agent_api::AgentApiPlugin;
use crate::civilization::CivilizationInputPlugin;
use crate::civilization::components::*;
use crate::civilization::concepts::*;
use crate::civilization::enums::GameFaction;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::game_moves::GameMovesPlugin;
use crate::civilization::general_systems::{
    connect_areas, fix_token_positions, move_tokens_from_stock_to_area, print_names_of_phases,
    start_game,
};
use crate::civilization::resolve_calamities::resolve_calamities_plugin::ResolveCalamitiesPlugin;
use crate::civilization::triggers::{on_add_return_city_to_stock, on_add_return_token_to_stock};
use crate::player::Player;
use crate::stupid_ai::*;
use crate::{GameActivity, GameState};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{AppExtStates, IntoScheduleConfigs, OnEnter, Res, Resource, in_state};

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CivLogicPlugins).add_plugins((
            CivilizationInputPlugin,
            TradeUiPlugin,
            AreaInfoPlugin,
            lava_ui_builder::LavaUiPlugin,
            AgentApiPlugin,
        ));
    }
}

/// The rules engine without any UI shell: everything needed to *play* the
/// game (phases, moves, AI, map, save), but none of the plugins that need a
/// window, input devices or rendered UI. The headless multiplayer server
/// builds on exactly this; `CivilizationPlugin` adds the UI shell on top.
///
/// Note: several phase plugins still contain UI *systems* (highlights, HUD
/// panels). Those are inert headless as long as the resources they read
/// exist — the server inserts inert `ButtonInput`/`TextureAssets`/`LavaTheme`
/// stand-ins. Splitting those systems out per-plugin is follow-up work.
pub struct CivLogicPlugins;

impl Plugin for CivLogicPlugins {
    fn build(&self, app: &mut App) {
        // Use DebugOptions::test_manual_pop_exp() to test manual population expansion
        app.insert_resource(DebugOptions::from_env())
            .register_type::<Player>()
            .register_type::<BarbarianToken>()
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
            .register_type::<SeaPassage>()
            .register_type::<OpenSea>()
            .register_type::<AstPosition>()
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
                TaxationPlugin,
                ShipsPlugin,
                SuccessionPlugin,
                PopulationExpansionPlugin,
                CensusPlugin,
                MovementPlugin,
                ConflictPlugin,
                TradePlugin,
                CivCardsPlugin,
            ))
            .add_plugins((
                CityConstructionPlugin,
                RemoveSurplusPlugin,
                ShedCommodityCardsPlugin,
                ResolveCalamitiesPlugin,
                CitySupportPlugin,
                StupidAiPlugin,
                GameMovesPlugin,
                TradeCardPlugin,
                MapPlugin,
                SaveGamePlugin,
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
            .add_observer(on_add_return_token_to_stock)
            .add_observer(on_add_return_city_to_stock);
    }
}

#[derive(Resource)]
// A bag of debug toggles; bools are the whole point.
#[allow(clippy::struct_excessive_bools)]
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
    /// If set, every AI player gets this playstyle instead of the round-robin
    /// spread over all archetypes (for isolating/testing one personality).
    pub force_playstyle: Option<Playstyle>,
    /// Watch mode: keep the whole map framed and suppress all automatic camera
    /// panning/focusing, so you can watch the AI play without the view jumping
    /// around. Manual zoom/pan keys still work.
    pub static_map_view: bool,
    /// Factions reserved for network seats: `setup_players` includes these
    /// first (before the local human / random fill), so the multiplayer
    /// server knows which factions its seats map to.
    pub reserved_factions: Vec<GameFaction>,
}

/// Run condition: automatic camera panning/focusing is enabled (i.e. not in the
/// static, whole-map watch mode).
pub fn camera_auto_pan_enabled(debug: Res<DebugOptions>) -> bool {
    !debug.static_map_view
}

/// Run condition: the whole-map watch mode is on.
pub fn static_map_view_enabled(debug: Res<DebugOptions>) -> bool {
    debug.static_map_view
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            add_human_player: true,
            human_faction: GameFaction::Egypt,
            human_always_pulls_trade_cards: false,
            ai_always_pulls_trade_cards: false,
            human_starts_with_trade_cards: false,
            auto_trading: true,
            print_selected_moves: true,
            log_selected_moves: false,
            number_of_players: 8,
            start_at_activity: None,
            human_token_count: None,
            human_starting_areas: None,
            specific_state_name: None,
            ai_move_delay_secs: 0.01,
            show_debug_ui: true,
            human_trade_cards: None,
            human_civ_cards: None,
            force_playstyle: None,
            static_map_view: false,
            reserved_factions: Vec::new(),
        }
    }
}

impl DebugOptions {
    /// Start from the defaults and apply command-line env overrides:
    ///
    /// - `NUM_PLAYERS=<n>` — total number of players (clamped to 1..=9, the number
    ///   of available factions).
    /// - `HUMAN_PLAYER=0|false|no` — drop the local human (full self-play); any
    ///   other value (or unset) keeps the default human player.
    ///
    /// Orthogonal to `AGENT_FACTIONS`, which decides which non-human factions are
    /// agent-controlled. E.g. `NUM_PLAYERS=4 HUMAN_PLAYER=0 AGENT_FACTIONS=all`
    /// is a 4-player, all-agent game.
    pub fn from_env() -> Self {
        let mut opts = Self::default();
        if let Ok(n) = std::env::var("NUM_PLAYERS")
            && let Ok(n) = n.trim().parse::<usize>()
        {
            opts.number_of_players = n.clamp(1, 9);
        }
        if let Ok(v) = std::env::var("HUMAN_PLAYER") {
            let v = v.trim();
            opts.add_human_player = !matches!(v, "0" | "false" | "no" | "off");
        }
        if let Ok(v) = std::env::var("STATIC_MAP") {
            let v = v.trim();
            opts.static_map_view = !matches!(v, "0" | "false" | "no" | "off");
        }
        opts
    }

    /// Create a debug configuration for testing manual population expansion.
    /// This gives the human player limited tokens and multiple populated areas.
    pub fn test_civ_cards() -> Self {
        Self {
            add_human_player: false,
            human_faction: GameFaction::Babylon,
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
            force_playstyle: None,
            static_map_view: false,
            reserved_factions: Vec::new(),
        }
    }
}
