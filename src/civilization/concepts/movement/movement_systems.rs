use crate::civilization::components::{Faction, FixTokenPositions, PlayerAreas, Population, Token};
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::movement::movement_components::*;
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::civilization::game_moves::{AvailableMoves, RecalculatePlayerMoves};
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{
    info, Commands, Entity, MessageReader, MessageWriter, Name, NextState, Query, Res, ResMut, Transform, With, Without,
};
use bevy::time::Time;

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
    mut next_player: MessageWriter<NextPlayerStarted>,
    loading_from_save: Option<Res<LoadingFromSave>>,
    faction_query: Query<(Entity, &Faction), With<Player>>,
    mut commands: Commands,
) {
    if let Some(ref save_state) = loading_from_save {
        // Restore census_order and left_to_move from save data
        // Resolve faction names back to entities
        let faction_to_entity: bevy::platform::collections::HashMap<_, _> = faction_query
            .iter()
            .map(|(e, f)| (f.faction, e))
            .collect();
        
        game_info.census_order = save_state.census_order.iter()
            .filter_map(|f| faction_to_entity.get(f).copied())
            .collect();
        game_info.left_to_move = save_state.left_to_move.iter()
            .filter_map(|f| faction_to_entity.get(f).copied())
            .collect();
        
        // Restore the player who was actively moving (already popped from left_to_move)
        if let Some(ref mover_faction) = save_state.current_mover {
            if let Some(&mover_entity) = faction_to_entity.get(mover_faction) {
                info!("[MOVEMENT] Restoring current mover: {:?}", mover_faction);
                commands.entity(mover_entity).insert(PerformingMovement);
            } else {
                info!("[MOVEMENT] No current mover to restore, advancing to next player");
                next_player.write(NextPlayerStarted);
            }
        } else {
            info!("[MOVEMENT] No current mover in save, advancing to next player");
            next_player.write(NextPlayerStarted);
        }
        
        info!("[MOVEMENT] Restored from save: {} in census_order, {} left to move",
            game_info.census_order.len(), game_info.left_to_move.len());
        
        commands.remove_resource::<LoadingFromSave>();
    } else {
        game_info.left_to_move = game_info.census_order.clone();
        game_info.left_to_move.reverse();
        next_player.write(NextPlayerStarted);
    }
}

pub fn prepare_next_mover(
    mut started: MessageReader<NextPlayerStarted>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    _names: Query<&Name>,
) {
    for _ in started.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
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
    token_transform: Query<&Transform, With<Token>>,
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
                            let target_pos = target_transform.translation;
                            tokens_to_move.iter().for_each(|token| {
                                commands.entity(*token).insert(TokenHasMoved);
                                if let Ok(token_transform) = token_transform.get(*token) {
                                    let start_pos = token_transform.translation;
                                    commands.entity(*token).insert(TokenMoveAnimation::new(
                                        start_pos,
                                        target_pos,
                                        0.15, // 150ms animation duration
                                    ));
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

/// System to animate tokens moving from one area to another
pub fn animate_token_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TokenMoveAnimation)>,
) {
    for (entity, mut transform, mut animation) in query.iter_mut() {
        animation.elapsed += time.delta_secs();
        transform.translation = animation.current_position();
        
        if animation.is_complete() {
            // Ensure we're exactly at the end position
            transform.translation = animation.end_pos;
            commands.entity(entity).remove::<TokenMoveAnimation>();
        }
    }
}
