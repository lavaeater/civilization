use crate::GameActivity;
use crate::civilization::PlayerShips;
use crate::civilization::components::{
    Faction, FixTokenPositions, GameArea, PlayerAreas, Population, Token,
};
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::map::CameraFocusQueue;
use crate::civilization::concepts::movement::movement_components::*;
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::civilization::game_moves::{AvailableMoves, RecalculatePlayerMoves};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use bevy::prelude::{
    Commands, Entity, Has, MessageReader, MessageWriter, Name, NextState, Query, Res, ResMut,
    Transform, With, Without, info,
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
        let faction_to_entity: bevy::platform::collections::HashMap<_, _> =
            faction_query.iter().map(|(e, f)| (f.faction, e)).collect();

        game_info.census_order = save_state
            .census_order
            .iter()
            .filter_map(|f| faction_to_entity.get(f).copied())
            .collect();
        game_info.left_to_move = save_state
            .left_to_move
            .iter()
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

        info!(
            "[MOVEMENT] Restored from save: {} in census_order, {} left to move",
            game_info.census_order.len(),
            game_info.left_to_move.len()
        );

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
        let name = names
            .get(end_movement_event.player)
            .map(|n| n.as_str())
            .unwrap_or("?");
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
    mut pop_query: Query<
        (
            &mut Population,
            &Transform,
            Option<&Name>,
            Option<&GameArea>,
        ),
        Without<Token>,
    >,
    mut commands: Commands,
    mut player_areas: Query<&mut PlayerAreas>,
    tokens_that_can_move: Query<&Token, Without<TokenHasMoved>>,
    token_transform: Query<&Transform, With<Token>>,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
    human_query: Query<Entity, With<IsHuman>>,
    player_is_human: Query<Has<IsHuman>>,
    mut camera_focus: ResMut<CameraFocusQueue>,
) {
    let human_player = human_query.iter().next();

    for ev in move_events.read() {
        if let Ok((mut from_pop, _, _, _)) = pop_query.get_mut(ev.source_area) {
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

                    if let Ok((mut to_pop, target_transform, area_name, game_area)) =
                        pop_query.get_mut(ev.target_area)
                    {
                        // Check if AI is moving into an area with human player tokens
                        let mover_is_ai =
                            player_is_human.get(ev.player).map(|h| !h).unwrap_or(true);
                        let human_has_tokens =
                            human_player.map(|h| to_pop.has_player(&h)).unwrap_or(false);

                        if mover_is_ai && human_has_tokens {
                            let area_desc = area_name
                                .map(|n| n.to_string())
                                .or_else(|| game_area.map(|a| format!("Area {}", a.id)))
                                .unwrap_or_else(|| "unknown area".to_string());
                            camera_focus.add_focus(
                                target_transform.translation,
                                1.0,
                                format!("AI moving into {}", area_desc),
                            );
                        }

                        if let Ok(mut player_area) = player_areas.get_mut(ev.player) {
                            let target_pos = target_transform.translation;
                            tokens_to_move.iter().for_each(|token| {
                                commands.entity(*token).insert(TokenHasMoved);
                                if let Ok(token_transform) = token_transform.get(*token) {
                                    let start_pos = token_transform.translation;
                                    commands.entity(*token).insert(TokenMoveAnimation::new(
                                        start_pos, target_pos,
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

/// Executes a ship ferry: moves up to 5 unmoved tokens from `source_area` to
/// `target_area` via sea passage, and advances the ship to the target area.
///
/// The ship moves one sea hop per command (rule 23.52 — up to 4 hops total per phase).
/// Tokens are marked `TokenHasMoved` so they cannot move again this phase (rule 23.51).
pub fn execute_ship_ferry(
    mut ferry_events: MessageReader<ShipFerryCommand>,
    mut pop_query: Query<(&mut Population, &Transform), Without<Token>>,
    mut player_ships_query: Query<&mut PlayerShips>,
    mut player_areas_query: Query<&mut PlayerAreas>,
    tokens_that_can_move: Query<&Token, Without<TokenHasMoved>>,
    token_transform: Query<&Transform, With<Token>>,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
    mut commands: Commands,
) {
    for ev in ferry_events.read() {
        // Collect tokens to move (unmoved, belonging to this player, in source area).
        let tokens_to_ferry: Vec<Entity> = {
            let Ok((pop, _)) = pop_query.get(ev.source_area) else {
                continue;
            };
            let Some(player_tokens) = pop.player_tokens().get(&ev.player) else {
                continue;
            };
            player_tokens
                .iter()
                .filter(|&&t| tokens_that_can_move.get(t).is_ok())
                .take(ev.number_of_tokens)
                .copied()
                .collect()
        };

        if tokens_to_ferry.is_empty() {
            recalculate_player_moves.write(RecalculatePlayerMoves::new(ev.player));
            continue;
        }

        // Remove tokens from source population.
        if let Ok((mut from_pop, _)) = pop_query.get_mut(ev.source_area) {
            for &token in &tokens_to_ferry {
                from_pop.remove_token_from_area(ev.player, token);
            }
        }

        // Get target area transform for animation end-point.
        let target_pos = pop_query
            .get(ev.target_area)
            .map(|(_, t)| t.translation)
            .unwrap_or_default();

        // Add tokens to target population and mark as moved.
        if let Ok((mut to_pop, _)) = pop_query.get_mut(ev.target_area) {
            for &token in &tokens_to_ferry {
                to_pop.add_token_to_area(ev.player, token);
            }
        }

        // Update PlayerAreas and animate tokens.
        if let Ok(mut player_areas) = player_areas_query.get_mut(ev.player) {
            for &token in &tokens_to_ferry {
                player_areas.remove_token_from_area(&ev.source_area, token);
                player_areas.add_token_to_area(ev.target_area, token);

                commands.entity(token).insert(TokenHasMoved);
                if let Ok(start) = token_transform.get(token) {
                    commands.entity(token).insert(TokenMoveAnimation::new(
                        start.translation,
                        target_pos,
                        0.25, // slightly longer than land movement to feel different
                    ));
                }
            }
        }

        // Move the ship from source to target in PlayerShips.
        if let Ok(mut ships) = player_ships_query.get_mut(ev.player)
            && let Some(ship_entity) = ships.remove_ship_from_area(ev.source_area)
        {
            ships.place_ship(ev.target_area, ship_entity);
            // Move the ship sprite to the target area position.
            commands
                .entity(ship_entity)
                .insert(Transform::from_translation(
                    target_pos + bevy::math::Vec3::new(8.0, 8.0, 2.0),
                ));
            info!(
                "[SHIPS] Ferry: {} token(s) moved {:?} → {:?}; ship follows",
                tokens_to_ferry.len(),
                ev.source_area,
                ev.target_area
            );
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
