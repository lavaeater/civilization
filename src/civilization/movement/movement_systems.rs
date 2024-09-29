use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::general::general_components::{PlayerAreas, Population, Token};
use crate::civilization::movement::movement_events::MoveTokenFromAreaToAreaCommand;
use crate::civilization::movement::movement_events::{NextPlayerStarted, PlayerMovementEnded};
use crate::GameActivity;
use bevy::prelude::{debug, Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With, Without};
use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::movement::movement_components::{HasJustMoved, PerformingMovement, TokenHasMoved};
use crate::player::Player;

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
    mut next_player: EventWriter<NextPlayerStarted>,
) {
    game_info.left_to_move = game_info.census_order.clone();
    game_info.left_to_move.reverse();
    next_player.send(NextPlayerStarted);
}

pub fn prepare_next_mover(
    mut started: EventReader<NextPlayerStarted>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in started.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            // debug!("Player {} is moving", to_move);
            // debug!("Before adding components");
            // commands.entity(to_move).log_components();
            commands.entity(to_move).insert(PerformingMovement);
            // commands.entity(to_move).log_components();
        } else {
            // debug!("All hath moved");
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
    mut end_event: EventReader<PlayerMovementEnded>,
    mut commands: Commands,
    mut next_player: EventWriter<NextPlayerStarted>,
) {
    for end_movement_event in end_event.read() {
        // debug!("Player {} has ended movement", end_movement_event.player);
        // commands.entity(end_movement_event.player).log_components();
        commands.entity(end_movement_event.player).remove::<PerformingMovement>();
        commands.entity(end_movement_event.player).remove::<AvailableMoves>();
        // debug!("After removing components");
        // commands.entity(end_movement_event.player).log_components();
        next_player.send(NextPlayerStarted);
    }
}

pub fn move_tokens_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToAreaCommand>,
    mut pop_query: Query<&mut Population>,
    mut commands: Commands,
    mut player_areas: Query<&mut PlayerAreas>,
    tokens_that_can_move: Query<&Token, Without<TokenHasMoved>>,
    mut recalculate_player_moves: EventWriter<RecalculatePlayerMoves>
) {
    for ev in move_events.read() {
        // debug!("Lets move some tokens!");
        if let Ok(mut from_pop) = pop_query.get_mut(ev.source_area) {
            let cloned = from_pop.player_tokens.clone();
            if let Some(mut player_tokens) = cloned.get(&ev.player) {
                // debug!("Player has tokens in the area");
                // debug!("Player tokens: {:?}", player_tokens);
                let tokens_that_can_move = player_tokens
                    .iter()
                    .filter(|t| tokens_that_can_move.get(**t).is_ok()).copied()
                    .collect::<Vec<_>>();
                if tokens_that_can_move.len() < ev.number_of_tokens {
                    // debug!("Not enough tokens to move, recalculate that son of a bitch!");
                    recalculate_player_moves.send(RecalculatePlayerMoves::new(ev.player));
                } else {
                    let tokens_to_move = tokens_that_can_move
                        .iter()
                        .take(ev.number_of_tokens).copied()
                        .collect::<Vec<_>>();
                    for token in tokens_to_move.iter() {
                        from_pop.remove_token_from_area(ev.player, *token);
                    }

                    if let Ok(mut to_pop) = pop_query.get_mut(ev.target_area) {
                        if let Ok(mut player_area) = player_areas.get_mut(ev.player) {
                            tokens_to_move
                                .iter()
                                .for_each(|token| {
                                    commands.entity(*token).insert(TokenHasMoved);

                                    player_area.remove_token_from_area(ev.source_area, *token);
                                    to_pop.add_token_to_area(ev.player, *token);
                                    player_area.add_token_to_area(ev.target_area, *token);

                                });
                        }
                    }
                }
            } else {
                // debug!("Player has no tokens in the area");
            }
            
        }
        commands.entity(ev.player).insert(HasJustMoved);
    }
}
