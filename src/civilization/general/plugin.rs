use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, Component, Entity, Event, IntoSystemConfigs, OnEnter, Reflect};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::civilization::census::plugin::CensusPlugin;
use crate::civilization::census::resources::GameInfoAndStuff;
use crate::civilization::game_phases::GamePhasesPlugin;
use crate::civilization::general::systems::{connect_areas, move_tokens_from_stock_to_area, setup_game};
use crate::civilization::movement::MovementPlugin;
use crate::civilization::population_expansion::PopulationExpansionPlugin;
use crate::GameState;

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Token>()
            .register_type::<LandPassage>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
            .add_event::<MoveTokenFromAreaToAreaCommand>()
            .add_plugins(
                (
                    MovementPlugin,
                    CensusPlugin,
                    GamePhasesPlugin,
                    PopulationExpansionPlugin
                )
            )
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_plugins(WorldInspectorPlugin::new())
            .insert_resource(GameInfoAndStuff::default())
            .add_systems(
                Update, (
                    connect_areas
                        .run_if(in_state(GameState::Playing)),
                    move_tokens_from_stock_to_area
                        .run_if(in_state(GameState::Playing))
                ));
    }
}

#[derive(Event, Debug)]
pub struct MoveTokensFromStockToAreaCommand {
    pub population_entity: Entity,
    pub stock_entity: Entity,
    pub number_of_tokens: usize,
}

#[derive(Event, Debug)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub from_area_population: Entity,
    pub to_area_population: Entity,
    pub tokens: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct Area {
    pub max_population: u8,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct LandPassage {
    pub to_areas: Vec<Entity>,
}

#[derive(Component, Debug, Reflect)]
pub struct NeedsConnections {
    pub land_connections: Vec<String>,
    pub sea_connections: Vec<String>,
}

#[derive(Component, Debug)]
pub struct Stock {
    pub max_tokens: usize,
    pub tokens: Vec<Entity>
}

impl Stock {
    pub(crate) fn new(max_tokens: usize, tokens: Vec<Entity>) -> Self {
        Stock {
            max_tokens,
            tokens
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Population {
    pub population: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct StartArea;

#[derive(Component, Debug, Reflect)]
pub struct Token {
    pub player: Entity,
}

#[derive(Component, Debug)]
pub struct CannotAutoExpandPopulation;
