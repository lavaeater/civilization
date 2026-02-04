use crate::civilization::components::population::Population;
use crate::civilization::components::*;
use crate::civilization::concepts::acquire_trade_cards::trade_card_plugin::TradeCardPlugin;
use crate::civilization::concepts::census::{CensusPlugin, GameInfoAndStuff};
use crate::civilization::concepts::check_city_support::check_city_support_plugin::CitySupportPlugin;
use crate::civilization::concepts::city_construction::city_construction_plugin::CityConstructionPlugin;
use crate::civilization::concepts::conflict::conflict_plugin::ConflictPlugin;
use crate::civilization::concepts::map::map_plugin::MapPlugin;
use crate::civilization::concepts::movement::movement_plugin::MovementPlugin;
use crate::civilization::concepts::population_expansion::population_expansion_plugin::PopulationExpansionPlugin;
use crate::civilization::concepts::remove_surplus_population::remove_surplus_plugin::RemoveSurplusPlugin;
use crate::civilization::concepts::trade::trade_plugin::TradePlugin;
use crate::civilization::enums::GameFaction;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::game_moves::game_moves_plugin::GameMovesPlugin;
use crate::civilization::general_systems::{connect_areas, fix_token_positions, move_tokens_from_stock_to_area, print_names_of_phases, start_game};
use crate::civilization::plugins::trade_ui_plugin::TradeUiPlugin;
use crate::civilization::triggers::on_add_return_token_to_stock;
use crate::stupid_ai::prelude::*;
use crate::{GameActivity, GameState};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, AppExtStates, IntoScheduleConfigs, OnEnter, Resource};

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugOptions::new(
            true,
            GameFaction::Assyria,
            true,
            false,
            false,
            false,
            false,
            7,
        ))
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
}

impl DebugOptions {
    pub fn new(
        add_human_player: bool,
        human_faction: GameFaction,
        human_always_pulls_trade_cards: bool,
        ai_always_pulls_trade_cards: bool,
        human_starts_with_trade_cards: bool,
        auto_trading: bool,
        print_selected_moves: bool,
        number_of_players: usize,
    ) -> Self {
        Self {
            add_human_player,
            human_faction,
            human_always_pulls_trade_cards,
            ai_always_pulls_trade_cards,
            human_starts_with_trade_cards,
            auto_trading,
            print_selected_moves,
            number_of_players,
        }
    }
}
