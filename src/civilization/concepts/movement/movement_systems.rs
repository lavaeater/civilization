use crate::civilization::components::population::Population;
use crate::civilization::components::{FixTokenPositions, PlayerAreas, Token};
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::movement::movement_components::*;
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{
    Commands, Entity, MessageReader, MessageWriter, NextState, Query, ResMut, Transform, With, Without,
};

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
    mut next_player: MessageWriter<NextPlayerStarted>,
) {
    game_info.left_to_move = game_info.census_order.clone();
    game_info.left_to_move.reverse();
    next_player.write(NextPlayerStarted);
}

pub fn prepare_next_mover(
    mut started: MessageReader<NextPlayerStarted>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in started.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            commands.entity(to_move).insert(PerformingMovement);
        } else {
            next_state.set(GameActivity::Conflict);
        }
    }
}

pub fn on_exit_movement(
    token_query: Query<Entity, With<TokenHasMoved>>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for player in player_query.iter() {
        commands.entity(player).remove::<PerformingMovement>();
    }

    for token in token_query.iter() {
        commands.entity(token).remove::<TokenHasMoved>();
    }
}

pub fn player_end_movement(
    mut end_event: MessageReader<PlayerMovementEnded>,
    mut commands: Commands,
    mut next_player: MessageWriter<NextPlayerStarted>,
) {
    for end_movement_event in end_event.read() {
        // //debug!("Player {} has ended movement", end_movement_event.player);
        // commands.entity(end_movement_event.player).log_components();
        commands
            .entity(end_movement_event.player)
            .remove::<PerformingMovement>();
        commands
            .entity(end_movement_event.player)
            .remove::<AvailableMoves>();
        // //debug!("After removing components");
        // commands.entity(end_movement_event.player).log_components();
        next_player.write(NextPlayerStarted);
    }
}

pub fn move_tokens_from_area_to_area(
    mut move_events: MessageReader<MoveTokenFromAreaToAreaCommand>,
    mut pop_query: Query<(&mut Population, &Transform), Without<Token>>,
    mut commands: Commands,
    mut player_areas: Query<&mut PlayerAreas>,
    tokens_that_can_move: Query<&Token, Without<TokenHasMoved>>,
    mut token_transform: Query<&mut Transform, With<Token>>,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
) {
    for ev in move_events.read() {
        if let Ok((mut from_pop, _)) = pop_query.get_mut(ev.source_area) {
            let cloned = from_pop.player_tokens().clone();
            if let Some(player_tokens) = cloned.get(&ev.player) {
                let tokens_that_can_move = player_tokens
                    .iter()
                    .filter(|t| tokens_that_can_move.get(**t).is_ok())
                    .copied()
                    .collect::<Vec<_>>();
                if tokens_that_can_move.len() < ev.number_of_tokens {
                    recalculate_player_moves.write(RecalculatePlayerMoves::new(ev.player));
                } else {
                    let tokens_to_move = tokens_that_can_move
                        .iter()
                        .take(ev.number_of_tokens)
                        .copied()
                        .collect::<Vec<_>>();
                    for token in tokens_to_move.iter() {
                        from_pop.remove_token_from_area(ev.player, *token);
                    }

                    if let Ok((mut to_pop, target_transform)) = pop_query.get_mut(ev.target_area) {
                        if let Ok(mut player_area) = player_areas.get_mut(ev.player) {
                            tokens_to_move.iter().for_each(|token| {
                                commands.entity(*token).insert(TokenHasMoved);
                                if let Ok(mut token_transform) = token_transform.get_mut(*token) {
                                    token_transform.translation = target_transform.translation;
                                }
                                player_area.remove_token_from_area(&ev.source_area, *token);
                                to_pop.add_token_to_area(ev.player, *token);
                                player_area.add_token_to_area(ev.target_area, *token);
                            });
                        }
                    }
                }
            }
        }
        commands.entity(ev.player).insert(HasJustMoved);
        commands.entity(ev.source_area).insert(FixTokenPositions);
        commands.entity(ev.target_area).insert(FixTokenPositions);
    }
}
