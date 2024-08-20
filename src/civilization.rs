use bevy::gizmos::start_gizmo_context;
use crate::player::Player;
use bevy::prelude::{BuildChildren, Children, Commands, Component, Entity, Event, EventReader, EventWriter, Parent, Query, With, Without};
use bevy::utils::HashMap;
use itertools::Itertools;

#[derive(Event, Debug)]
struct BeginPopulationExpansion;

#[derive(Event, Debug)]
struct CheckPopulationExpansionEligibility;

#[derive(Event, Debug)]
struct MoveTokensFromStockToArea {
    pub area_entity: Entity,
    pub player_entity: Entity,
    pub number_of_tokens: u8,
}

#[derive(Event, Debug)]
struct MoveTokenFromAreaToArea {
    pub from_area: Entity,
    pub to_area: Entity,
    pub tokens: Vec<Entity>,
}

#[derive(Event, Debug)]
struct CheckAreaSurplus {
    pub area: Entity,
}

#[derive(Component, Debug)]
struct Area {
    pub max_population: u8,
}

#[derive(Component, Debug)]
struct Stock;

#[derive(Component, Debug)]
struct Population;

#[derive(Component, Debug)]
struct Token {
    pub player: Entity,
}

/***
Is it reasonable to have a marker component for this or will that add complexity somewhere
else in the game? I don't knooow
 */
#[derive(Component, Debug)]
struct AreaHasPopulation;

#[derive(Component, Debug)]
struct AreaHasSurplusPopulation;

#[derive(Component, Debug)]
struct CannotAutoExpandPopulation;

fn move_token_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToArea>,
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
    mut begin_event: EventReader<CheckPopulationExpansionEligibility>,
    area_population_query: Query<&Children, With<Population>>,
    token_query: Query<&Token>,
    player_stock_query: Query<&Children, With<Stock>>,
    player_query: Query<&Children, With<Player>>,
    mut commands: Commands
) {
    for _event in begin_event.read() {
        let mut player_need_tokens_hash = HashMap::<Entity, usize>::new();
        for (area_pop_tokens) in area_population_query.iter() {
            if area_pop_tokens.iter().count() > 0 {
                for (player_entity, player_area_tokens) in area_pop_tokens
                    .iter()
                    .chunk_by(|pop_ent| {
                        token_query.get(**pop_ent).unwrap().player
                    }).into_iter() {
                    let mut required_player_tokens = 0;
                    let c = player_area_tokens.count();

                    required_player_tokens += if c == 1 {
                        1
                    } else if c > 1 {
                        2
                    } else {
                        0
                    };
                    if player_need_tokens_hash.contains_key(&player_entity) {
                        player_need_tokens_hash[player_entity] = player_need_tokens_hash[player_entity] + c;
                    } else {
                        player_need_tokens_hash[player_entity] = c;
                    }
                }
            }
        }
        for (player, needed_tokens) in player_need_tokens_hash {
            if let Ok(children) = player_query.get(player) {
                for child in children {
                    if let Ok(tokens) = player_stock_query.get(*child) {
                        if tokens.iter().count() < needed_tokens {
                            commands.entity(player).add(CannotAutoExpandPopulation);
                        }
                    }
                }
            }
        }
    }
}

fn expand_population(
    mut begin_event: EventReader<BeginPopulationExpansion>,
    area_query: Query<(Entity, &Children), With<Population>>,
    token_query: Query<&Token>,
    player_eligible_query: Query<&Player, Without<CannotAutoExpandPopulation>>,
    mut event_writer: EventWriter<MoveTokensFromStockToArea>,
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
                        token_query.get(**pop_ent).unwrap().player
                    }).into_iter() {
                    if player_eligible_query.get(player).is_ok() {
                        let c = tokens.count();
                        if c == 1 {
                            event_writer.send(MoveTokensFromStockToArea {
                                area_entity: area_population_entity,
                                player_entity: player,
                                number_of_tokens: 1,
                            });
                        } else if c > 1 {
                            event_writer.send(MoveTokensFromStockToArea {
                                area_entity: area_population_entity,
                                player_entity: player,
                                number_of_tokens: 2,
                            });
                        }
                    }
                }
            }
        }
    }
}


fn remove_surplus_population(query: Query<&Area, With<AreaHasSurplusPopulation>>) {}