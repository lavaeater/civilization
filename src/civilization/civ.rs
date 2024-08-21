use bevy::app::{App, Plugin, Update};
use crate::player::Player;
use bevy::prelude::{in_state, info, BuildChildren, Children, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Name, OnEnter, Query, Reflect, With, Without};
use bevy::utils::HashMap;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use itertools::Itertools;
use crate::GameState;

pub struct CivilizationPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Token>()
            .add_event::<BeginPopulationExpansionEvent>()
            .add_event::<CheckPopulationExpansionEligibilityEvent>()
            .add_event::<StartManualPopulationExpansionEvent>()
            .add_event::<StartHandleSurplusPopulationEvent>()
            .add_event::<MoveTokensFromStockToAreaCommand>()
            .add_event::<MoveTokenFromAreaToAreaCommand>()
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_plugins(WorldInspectorPlugin::new())
            .add_systems(
                Update, (
                    move_token_from_area_to_area
                        .run_if(in_state(GameState::Playing)),
                    handle_manual_population_expansion
                        .run_if(in_state(GameState::Playing)),
                    check_population_expansion_eligibility
                        .run_if(in_state(GameState::Playing)),
                    expand_population
                        .run_if(in_state(GameState::Playing)),
                    move_tokens_from_stock_to_area
                        .run_if(in_state(GameState::Playing))
                ));
    }
}

#[derive(Event, Debug)]
pub struct BeginPopulationExpansionEvent;

#[derive(Event, Debug)]
pub struct CheckPopulationExpansionEligibilityEvent;

#[derive(Event, Debug)]
pub struct StartManualPopulationExpansionEvent;

#[derive(Event, Debug)]
pub struct StartHandleSurplusPopulationEvent;

#[derive(Event, Debug)]
pub struct MoveTokensFromStockToAreaCommand {
    pub population_entity: Entity,
    pub stock_entity: Entity,
    pub number_of_tokens: usize,
}

#[derive(Event, Debug)]
pub struct MoveTokenFromAreaToAreaCommand {
    pub from_area: Entity,
    pub to_area: Entity,
    pub tokens: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct Area {
    pub max_population: u8,
}

#[derive(Component, Debug)]
pub struct Stock;

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
    // Create Player
    let player = commands
        .spawn((Player {}, Name::new("Player")))
        .id();

    let stock = commands.spawn((Stock {}, Name::new("Stock"))).id();

    commands.entity(player).add_child(stock);

    for _n in 0..51 {
        let token = commands.spawn((Name::new("Token"), Token { player })).id();
        commands.entity(stock).add_child(token);
    }

    // Create some Areas
    commands
        .spawn((
            Name::new("Start Area"),
            Area { max_population: 2 },
            StartArea {}))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn((Area { max_population: 3 }, Name::new("Area two")))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn((Area { max_population: 1 }, Name::new("Area three")))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
    commands
        .spawn((Area { max_population: 5 }, Name::new("Area four")))
        .with_children(|c| { c.spawn((Name::new("Population"), Population {})); });
}

fn move_token_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToAreaCommand>,
    mut commands: Commands,
) {
    for ev in move_events.read() {
        commands.entity(ev.from_area).remove_children(&ev.tokens);
        commands.entity(ev.to_area).push_children(&ev.tokens);
    }
}

/***
A system that checks if an area has children... I mean, this is completely unnecessary really
 */
fn check_population_expansion_eligibility(
    mut begin_event: EventReader<CheckPopulationExpansionEligibilityEvent>,
    area_population_query: Query<&Children, With<Population>>,
    token_query: Query<&Token>,
    player_stock_query: Query<&Children, With<Stock>>,
    player_query: Query<&Children, With<Player>>,
    mut commands: Commands,
    mut start_manual_expansion: EventWriter<StartManualPopulationExpansionEvent>,
    mut start_pop_expansion: EventWriter<BeginPopulationExpansionEvent>,
) {
    for _event in begin_event.read() {
        let mut player_need_tokens_hash = HashMap::<Entity, usize>::new();
        for area_pop_tokens in area_population_query.iter() {
            if area_pop_tokens.iter().count() > 0 {
                for (player_entity, player_area_tokens) in area_pop_tokens
                    .iter()
                    .chunk_by(|pop_ent| {
                        token_query.get(**pop_ent).unwrap().player
                    }).into_iter() {
                    let mut required_player_tokens = 0;
                    let c = player_area_tokens.count();

                    required_player_tokens += match c {
                        1 => { 1 }
                        0 => { 0 }
                        _ => { 2 }
                    };
                    *player_need_tokens_hash.entry(player_entity).or_insert(0) += required_player_tokens;
                }
            }
        }
        let mut need_manual_expansion = false;
        for (player, needed_tokens) in player_need_tokens_hash {
            if let Ok(children) = player_query.get(player) {
                for child in children {
                    if let Ok(tokens) = player_stock_query.get(*child) {
                        if tokens.iter().count() < needed_tokens {
                            need_manual_expansion = true;
                            commands
                                .entity(player)
                                .insert(CannotAutoExpandPopulation {});
                        }
                    }
                }
            }
        }
        if need_manual_expansion {
            start_manual_expansion.send(StartManualPopulationExpansionEvent {});
        } else {
            start_pop_expansion.send(BeginPopulationExpansionEvent {});
        }
    }
}

fn handle_manual_population_expansion(
    mut start_reader: EventReader<StartManualPopulationExpansionEvent>,
    mut expand_writer: EventWriter<BeginPopulationExpansionEvent>,
) {
    for _start in start_reader.read() {
        expand_writer.send(BeginPopulationExpansionEvent {});
    }
}

fn expand_population(
    mut begin_event: EventReader<BeginPopulationExpansionEvent>,
    area_query: Query<(Entity, &Children), With<Population>>,
    token_query: Query<&Token>,
    player_eligible_query: Query<&Player, Without<CannotAutoExpandPopulation>>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
) {
    /*
    But what do we do in the case of the player not having enough tokens to expand the population
    across all his areas?

    This needs simply to be a special case that we examine before this system is run.

    Ah, it is even preferrable to do this only for players that are in fact able to expand their population
     */

    for _event in begin_event.read() {
        for (area_population_entity, area_population_tokens) in area_query.iter() {
            if area_population_tokens.iter().count() > 0 {
                for (player, tokens) in area_population_tokens
                    .iter()
                    .chunk_by(|pop_ent| {
                        let p = token_query.get(**pop_ent).unwrap().player;
                        info!("Found entity?");
                        p
                    }).into_iter() {
                    // if player_eligible_query.contains(player) {
                        let c = tokens.count();
                        match c {
                            0 => {}
                            1 => {
                                event_writer.send(MoveTokensFromStockToAreaCommand {
                                    population_entity: area_population_entity,
                                    stock_entity: player,
                                    number_of_tokens: 1,
                                });
                            }
                            _ => {
                                event_writer.send(MoveTokensFromStockToAreaCommand {
                                    population_entity: area_population_entity,
                                    stock_entity: player,
                                    number_of_tokens: 2,
                                });
                            }
                        }
                    // }
                }
            }
        }
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