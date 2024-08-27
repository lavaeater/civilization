use crate::civilization::census::components::HasPopulation;
use crate::civilization::census::resources::GameInfoAndStuff;
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::{LandPassage, Population};
use crate::civilization::movement::components::{MoveableTokens, PerformingMovement, TokenCanMove};
use crate::civilization::movement::events::EndPlayerMovement;
use crate::civilization::movement::events::MoveTokenFromAreaToAreaCommand;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, Name, NextState, Query, ResMut, With, Without};
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
) {
    game_info.left_to_move = game_info.census_order.clone();
    game_info.left_to_move.reverse();
}

pub fn prepare_next_mover(
    mut game_info: ResMut<GameInfoAndStuff>,
    moveable_tokens: Query<&Population, With<HasPopulation>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // If no one is moving, get poppin!
    if game_info.current_mover.is_none() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            commands.entity(to_move).insert(PerformingMovement {});
            game_info.current_mover = Some(to_move);
            moveable_tokens.iter().for_each(|population| {
                if population.player_tokens.contains_key(&to_move) {
                    for token in population.player_tokens[&to_move].iter() {
                        commands.entity(*token).insert(TokenCanMove {});
                    }
                }
            });
        } else {
            // All hath moved, move along
            next_state.set(GameActivity::Conflict);
        }
    }
}

pub fn clear_moves(
    areas_to_clear_query: Query<Entity, With<MoveableTokens>>,
    token_query: Query<Entity, With<TokenCanMove>>,
    mut commands: Commands,
) {
    for area in areas_to_clear_query.iter() {
        commands.entity(area).remove::<MoveableTokens>();
    }
    for token in token_query.iter() {
        commands.entity(token).remove::<TokenCanMove>();
    }
}

pub fn calculate_moves(
    moveable_tokens: Query<Entity, With<TokenCanMove>>,
    area_query: Query<(Entity, &LandPassage, &Population, &Name), Without<MoveableTokens>>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>,
) {
    for (area_entity, land_passage, population, name) in area_query.iter() {
        let area_tokens: Vec<Entity> =
            population
                .player_tokens
                .values()
                .flat_map(|v| v.iter()
                    .copied())
                .collect::<Vec<Entity>>();

        let area_moveable_tokens =
            area_tokens
                .iter()
                .filter(|t| moveable_tokens.contains(**t)).copied()
                .map(|t| t)
                .collect::<Vec<Entity>>();

        if area_moveable_tokens.len() > 0 {
            commands.entity(area_entity).insert(MoveableTokens {
                tokens: area_moveable_tokens,
                targets: land_passage.to_areas.clone(),
            });
        }
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Recalculated moves for {}", name))));
    }
}

pub fn player_end_movement(
    mut end_event: EventReader<EndPlayerMovement>,
    mut game_info_and_stuff: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
) {
    for _ in end_event.read() {
        if let Some(player) = game_info_and_stuff.current_mover {
            commands.entity(player).remove::<PerformingMovement>();
            game_info_and_stuff.current_mover = None;
        }
    }
}

pub fn move_token_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToAreaCommand>,
    mut pop_query: Query<&mut Population>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>,
) {
    for ev in move_events.read() {
        let mut tokens_to_move = vec![];
        if let Ok(mut from_pop) = pop_query.get_mut(ev.from_area) {
            tokens_to_move = (0..ev.number_of_tokens).map(|_| from_pop.player_tokens.get_mut(&ev.player).unwrap().swap_remove(0)).collect::<Vec<Entity>>();
        }
        if let Ok(mut to_pop) = pop_query.get_mut(ev.to_area) {
            tokens_to_move
                .iter()
                .for_each(|token| {
                    commands.entity(*token).remove::<TokenCanMove>();
                    to_pop
                        .player_tokens
                        .get_mut(&ev.player)
                        .unwrap()
                        .push(*token)
                });
            // this will make that area recompute its moves. Cool.
            commands.entity(ev.from_area).remove::<MoveableTokens>();
            write_line.send(PrintConsoleLine::new(StyledStr::from("Moved some tokens!")));
        }
    }
}
