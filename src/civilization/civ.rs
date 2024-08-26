use bevy::app::{App, Plugin, Update};
use crate::player::Player;
use bevy::prelude::{in_state, BuildChildren, Children, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Name, OnEnter, Query, Reflect, With};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::civilization::census::{Census, CensusPlugin, GameInfoAndStuff};
use crate::civilization::game_phases::GamePhasesPlugin;
use crate::civilization::movement;
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
            .register_type::<Census>()
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
                    movement::move_token_from_area_to_area
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
}

#[derive(Component, Debug)]
pub struct Population;

#[derive(Component, Debug)]
pub struct StartArea;

#[derive(Component, Debug, Reflect)]
pub struct Token {
    pub player: Entity,
}

#[derive(Component, Debug)]
pub struct CannotAutoExpandPopulation;

fn setup_game(
    mut commands: Commands,
) {
    (1..=1).into_iter().for_each(|n| {
        // Create Player
        let player = commands
            .spawn(
                (
                    Player {},
                    Name::new(format!("Player {n}")),
                    Census { population: 0 }
                )
            )
            .id();

        let stock = commands
            .spawn(
                (
                    Stock { max_tokens: 47 },
                    Name::new(format!("Stock {n}"))
                )
            )
            .id();

        commands
            .entity(player)
            .add_child(stock);

        for _n in 0..47 {
            let token = commands
                .spawn((
                    Name::new(format!("Token {n}")),
                    Token { player })).id();
            commands
                .entity(stock)
                .add_child(token);
        }
    });


    // Create some Areas
    commands
        .spawn(
            (
                Name::new("sa"),
                Area { max_population: 2 },
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("2".into(), "3".into()),
                    sea_connections: vec!(),
                },
                StartArea {}
            )
        )
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn(
            (
                Area { max_population: 3 },
                Name::new("2"),
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("sa".into(), "4".into()),
                    sea_connections: vec!(),
                },
            )
        )
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn(
            (
                Area { max_population: 1 },
                Name::new("3"),
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("sa".into(), "4".into()),
                    sea_connections: vec!(),
                },
            )
        )
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn(
            (
                Area { max_population: 5 },
                Name::new("4"),
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("2".into(), "3".into()),
                    sea_connections: vec!(),
                },
            )
        )
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
}

fn connect_areas(
    mut area_query: Query<(Entity, &mut LandPassage, &NeedsConnections)>,
    named_areas: Query<(Entity, &Name), With<Area>>,
    mut commands: Commands,
) {
    for (area_entity,
        mut land_passages,
        needed_connections) in area_query.iter_mut() {
        for named_area in needed_connections.land_connections.clone().into_iter() {
            let na = Name::new(named_area.clone());
            //This is fucking stupid, but who cares?
            for (target_area_entity, target_name) in named_areas.iter() {
                if *target_name == na {
                    land_passages.to_areas.push(target_area_entity);
                }
            }
        }
        commands.entity(area_entity).remove::<NeedsConnections>();
    }
}

/**
This is 100% needed to be able to test expansion and stuff.
*/
fn move_tokens_from_stock_to_area(
    mut move_commands: EventReader<MoveTokensFromStockToAreaCommand>,
    player_stock_query: Query<&Children, With<Stock>>,
    player_query: Query<&Children>,
    mut commands: Commands,
) {
    for ev in move_commands.read() {
        if let Ok(children) = player_query.get(ev.stock_entity) {
            for child in children {
                if let Ok(tokens) = player_stock_query.get(*child) {
                    let tokens_to_move = &tokens.into_iter().as_slice()[0..ev.number_of_tokens];
                    commands.entity(ev.stock_entity).remove_children(tokens_to_move);
                    commands.entity(ev.population_entity).push_children(tokens_to_move);
                }
            }
        }
    }
}