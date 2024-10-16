use crate::civilization::census::prelude::*;
use crate::civilization::city_construction::prelude::*;
use crate::civilization::city_support::prelude::*;
use crate::civilization::conflict::prelude::*;
use crate::civilization::console::prelude::*;
use crate::civilization::game_moves::prelude::*;
use crate::civilization::general::prelude::*;
use crate::civilization::map::map_plugin::MapPlugin;
use crate::civilization::movement::prelude::*;
use crate::civilization::population_expansion::prelude::*;
use crate::civilization::remove_surplus::prelude::*;
use crate::stupid_ai::prelude::*;
use crate::{GameActivity, GameState};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, AppExtStates, IntoSystemConfigs, OnEnter};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::civilization::trade_cards::prelude::*;

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Token>()
            .register_type::<LandPassage>()
            .register_type::<TokenStock>()
            .register_type::<GameArea>()
            .register_type::<Population>()
            .register_type::<PlayerAreas>()
            .register_type::<Faction>()
            .register_type::<Treasury>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
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
                    GameMovesPlugin,
                    TradeCardsPlugin,
                    MapPlugin,
                )
            )
            .add_systems(OnEnter(GameState::Playing), setup_players)
            .add_systems(OnEnter(GameActivity::StartGame), start_game)
            .add_plugins(WorldInspectorPlugin::new())
            .insert_resource(GameInfoAndStuff::default())
            .add_systems(
                Update, (
                    connect_areas
                        .run_if(in_state(GameState::Playing)),
                    move_tokens_from_stock_to_area
                        .run_if(in_state(GameState::Playing)),
                    fix_token_positions
                        .run_if(in_state(GameState::Playing)),
                ))
            .observe(on_add_return_token_to_stock)
        ;
    }
}

