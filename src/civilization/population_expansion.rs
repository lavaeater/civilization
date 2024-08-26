use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, Commands, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Query, With, Without};
use bevy::hierarchy::Children;
use bevy::log::info;
use bevy::utils::HashMap;
use itertools::Itertools;
use crate::civilization::game_phases_plugin::{GameActivity, GameActivityEnded, GameActivityStarted};
use crate::civilization::general::plugin::{Population, Stock, Token, CannotAutoExpandPopulation, MoveTokensFromStockToAreaCommand};
use crate::GameState;
use crate::player::Player;

pub struct PopulationExpansionPlugin;

impl Plugin for PopulationExpansionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PopulationExpansionEnded>()
            .add_event::<BeginPopulationExpansionEvent>()
            .add_event::<CheckPopulationExpansionEligibilityEvent>()
            .add_event::<StartManualPopulationExpansionEvent>()
            .add_systems(
                Update, (
                    handle_population_expansion_start
                        .run_if(in_state(GameState::Playing)),
                    handle_manual_population_expansion
                        .run_if(in_state(GameState::Playing)),
                    check_population_expansion_eligibility
                        .run_if(in_state(GameState::Playing)),
                    expand_population
                        .run_if(in_state(GameState::Playing)),
                    end_population_expansion
                        .run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug)]
pub struct BeginPopulationExpansionEvent;

#[derive(Event, Debug)]
pub struct CheckPopulationExpansionEligibilityEvent;

#[derive(Event, Debug)]
pub struct StartManualPopulationExpansionEvent;

#[derive(Event, Debug)]
pub struct PopulationExpansionEnded;

pub fn end_population_expansion(
    mut population_expansion_ended: EventReader<PopulationExpansionEnded>,
    mut game_activity_ended: EventWriter<GameActivityEnded>
) {
    for _ in population_expansion_ended.read() {
        game_activity_ended.send(GameActivityEnded(GameActivity::PopulationExpansion));
    }
}


pub fn handle_population_expansion_start(
    mut activity_event: EventReader<GameActivityStarted>,
    mut start_pop_exp: EventWriter<CheckPopulationExpansionEligibilityEvent>,
) {
    for activity in activity_event.read() {
        if activity.0 == GameActivity::PopulationExpansion {
            start_pop_exp.send(CheckPopulationExpansionEligibilityEvent {});
        }
    }
}

pub fn check_population_expansion_eligibility(
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

pub fn handle_manual_population_expansion(
    mut start_reader: EventReader<StartManualPopulationExpansionEvent>,
    mut expand_writer: EventWriter<BeginPopulationExpansionEvent>,
) {
    for _start in start_reader.read() {
        expand_writer.send(BeginPopulationExpansionEvent {});
    }
}

pub fn expand_population(
    mut begin_event: EventReader<BeginPopulationExpansionEvent>,
    area_query: Query<(Entity, &Children), With<Population>>,
    token_query: Query<&Token>,
    player_eligible_query: Query<&Player, Without<CannotAutoExpandPopulation>>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut end_population_expansion: EventWriter<PopulationExpansionEnded>,
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
                    if player_eligible_query.contains(player) {
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
                    }
                }
            }
        }
        end_population_expansion.send(PopulationExpansionEnded {});
    }
}