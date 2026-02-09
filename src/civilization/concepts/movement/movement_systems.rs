use crate::civilization::components::{Population, FixTokenPositions, PlayerAreas, Token};
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::movement::movement_components::*;
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::game_moves::{AvailableMoves, RecalculatePlayerMoves};
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{
    Commands, Entity, MessageReader, MessageWriter, Name, NextState, Query, ResMut, Transform, With, Without, info,
};

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
    mut next_player: MessageWriter<NextPlayerStarted>,
) {
    game_info.left_to_move = game_info.census_order.clone();
    game_info.left_to_move.reverse();
    info!("Starting movement activity with {} players", game_info.left_to_move.len());
    next_player.write(NextPlayerStarted);
}

pub fn prepare_next_mover(
    mut started: MessageReader<NextPlayerStarted>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    names: Query<&Name>,
) {
    for _ in started.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            let name = names.get(to_move).map(|n| n.as_str()).unwrap_or("?");
            info!("Preparing next mover: {}, {} remaining", name, game_info.left_to_move.len());
            commands.entity(to_move).insert(PerformingMovement);
        } else {
            info!("No more players to move, transitioning to Conflict");
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
    names: Query<&Name>,
) {
    for end_movement_event in end_event.read() {
        let name = names.get(end_movement_event.player).map(|n| n.as_str()).unwrap_or("?");
        info!("Player {} has ended movement", name);
        commands
            .entity(end_movement_event.player)
            .remove::<PerformingMovement>();
        commands
            .entity(end_movement_event.player)
            .remove::<AvailableMoves>();
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
