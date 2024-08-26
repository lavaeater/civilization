use bevy::app::{App, Update};
use bevy::hierarchy::{BuildChildren, Parent};
use crate::civilization::civ::{Area, LandPassage, MoveTokenFromAreaToAreaCommand, Population, Token};
use bevy::prelude::{in_state, Children, Commands, Component, Entity, Event, EventReader, EventWriter, HierarchyQueryExt, IntoSystemConfigs, Plugin, Query, Reflect, ResMut, With};
use bevy::utils::HashMap;
use itertools::Itertools;
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::census::{GameInfoAndStuff, HasPopulation};
use crate::civilization::game_phases::{GameActivity, GameActivityStarted};
use crate::GameState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PrepareNextMoverCommand>()
            .add_event::<CalculateMovesCommand>()
            .add_event::<ClearMovesCommand>()
            .add_systems(
                Update, (
                    start_movement_activity
                        .run_if(in_state(GameState::Playing)),
                    prepare_next_mover
                        .run_if(in_state(GameState::Playing)),
                    calculate_moves
                        .run_if(in_state(GameState::Playing)),
                    clear_moves
                        .run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug, Reflect)]
pub struct PrepareNextMoverCommand;

#[derive(Event, Debug, Reflect)]
pub struct ClearMovesCommand;

#[derive(Event, Debug, Reflect)]
pub struct CalculateMovesCommand;

#[derive(Component, Debug, Reflect)]
pub struct PerformingMovement;

#[derive(Component, Debug, Reflect)]
pub struct TokenCanMove;

#[derive(Component, Debug, Reflect)]
pub struct MoveableTokens {
    pub tokens: Vec<Entity>,
    pub targets: Vec<Entity>,
}

pub fn start_movement_activity(
    mut start_activity: EventReader<GameActivityStarted>,
    mut next_mover_command: EventWriter<PrepareNextMoverCommand>,
    mut game_info: ResMut<GameInfoAndStuff>,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Movement {
            game_info.left_to_move = game_info.census_order.clone();
            game_info.left_to_move.reverse();
            next_mover_command.send(PrepareNextMoverCommand {});
        }
    }
}

pub fn prepare_next_mover(
    mut next_mover: EventReader<PrepareNextMoverCommand>,
    mut game_info: ResMut<GameInfoAndStuff>,
    // mut move_ended_writer: EventWriter<GameActivityEnded>,
    moveable_tokens: Query<(&Population, &Children), With<HasPopulation>>,
    token_query: Query<&Token>,
    mut commands: Commands,
    mut clear_moves: EventWriter<ClearMovesCommand>,
) {
    for _ in next_mover.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            commands.entity(to_move).insert(PerformingMovement {});
            game_info.current_mover = Some(to_move);
            moveable_tokens.iter().for_each(|(_, tokens)| {
                for token_entity in tokens.iter() {
                    if let Ok(token) = token_query.get(*token_entity) {
                        if token.player == to_move {
                            commands.entity(*token_entity).insert(TokenCanMove {});
                        }
                    }
                }
            });
            clear_moves.send(ClearMovesCommand {});
        } else {
            // We're done bro
            game_info.current_mover = None;
            // move_ended_writer.send(GameActivityEnded(GameActivity::Movement));
        }
    }
}

pub fn clear_moves(
    mut clear_moves_reader: EventReader<ClearMovesCommand>,
    moveable_tokens: Query<(Entity, &MoveableTokens)>,
    mut commands: Commands,
    mut calculate_moves: EventWriter<CalculateMovesCommand>
) {
    for _ in clear_moves_reader.read() {
        moveable_tokens.iter().for_each(|(area, _)| {
            commands.entity(area).remove::<MoveableTokens>();
        });
        calculate_moves.send(CalculateMovesCommand {});
    }
}

pub fn calculate_moves(
    mut calc_moves_reader: EventReader<CalculateMovesCommand>,
    moveable_tokens: Query<(Entity, &Parent), With<TokenCanMove>>,
    parent: Query<&Parent>,
    area_query: Query<(&Area, &LandPassage)>,
    mut commands: Commands,
) {
    for _ in calc_moves_reader.read() {
        /*
        The goal here is to make it suuuper easy for the UI to show the moves.

        This means we should extract the parent (population) and the tokens themselves, into
        a hash and then what?
        We should add a component on area that holds the currently moveable tokens and potential
        targets to enable easy selection of movement.
         */

        // get all moveable tokens.
        // group by population (&parent)
        // count them
        //this is a useful hash, somehow
        let snake: HashMap<Entity, Vec<Entity>> = moveable_tokens
            .iter()
            .chunk_by(|(_, p)| {
                *p
            })
            .into_iter()
            .map(|(pop_entity, grouped_tokens)| {
                (pop_entity.get(), grouped_tokens.into_iter().map(|(token_entity, _)| token_entity).collect())
            }).collect()
            ;
        for (pop_ent, tokens) in snake {
            //find area, is top entity
            let area_entity: Entity = *parent
                .iter_ancestors(pop_ent)
                .filter(|e| {
                    area_query.contains(*e)
                })
                .collect::<Vec<Entity>>()
                .first()
                .unwrap();

            if tokens.len() > 0 {
                if let Ok((_area, lp)) = area_query.get(area_entity) {
                    commands.entity(area_entity).insert(MoveableTokens {
                        tokens,
                        targets: lp.to_areas.clone(),
                    });
                }
            }
        }
    }
}

pub fn move_token_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToAreaCommand>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>,
    mut calculate_moves_command: EventWriter<CalculateMovesCommand>
) {
    for ev in move_events.read() {
        ev.tokens.iter().for_each(|t| {
            commands.entity(*t).remove::<TokenCanMove>();
        });
        write_line.send(PrintConsoleLine::new(StyledStr::from("Moved some tokens!")));
        commands.entity(ev.from_area_population).remove_children(&ev.tokens);
        commands.entity(ev.to_area_population).push_children(&ev.tokens);
        calculate_moves_command.send(CalculateMovesCommand {});
    }
}