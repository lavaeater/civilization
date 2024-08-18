use crate::player::Player;
use bevy::prelude::{BuildChildren, Children, Commands, Component, Entity, Event, EventReader, EventWriter, Query, With};
use itertools::Itertools;

#[derive(Event, Debug)]
struct BeginPopulationExpansion;

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
fn check_population() {}

fn expand_population(
    mut begin_event: EventReader<BeginPopulationExpansion>,
    area_query: Query<(Entity, &Children), With<Population>>,
    token_query: Query<&Token>,
    mut event_writer: EventWriter<MoveTokensFromStockToArea>,
) {
    for _event in begin_event.read() {
        for (population_entity, population_tokens) in area_query.iter() {
            if population_tokens.iter().count() > 0 {
                for (player, tokens) in population_tokens
                    .iter()
                    .chunk_by(|pop_ent| {
                        token_query.get(**pop_ent).unwrap().player
                    }).into_iter() {
                    match tokens.count() {
                        1 => {
                            event_writer.send(MoveTokensFromStockToArea {
                                area_entity: population_entity,
                                player_entity: player,
                                number_of_tokens: 1,
                            });
                        }
                        2 => {
                            event_writer.send(MoveTokensFromStockToArea {
                                area_entity: population_entity,
                                player_entity: player,
                                number_of_tokens: 2,
                            });
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

fn remove_surplus_population(query: Query<&Area, With<AreaHasSurplusPopulation>>) {}