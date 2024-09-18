use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::general::general_components::{PlayerAreas, Population};
use crate::civilization::movement::movement_events::MoveTokenFromAreaToAreaCommand;
use crate::civilization::movement::movement_events::{NextPlayerStarted, PlayerMovementEnded};
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};
use crate::civilization::movement::movement_components::{PerformingMovement, TokenHasMoved};

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
    mut next_player: EventWriter<NextPlayerStarted>,
) {
    game_info.left_to_move = game_info.census_order.clone();
    game_info.left_to_move.reverse();
    next_player.send(NextPlayerStarted {});
}

pub fn prepare_next_mover(
    mut started: EventReader<NextPlayerStarted>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in started.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            commands.entity(to_move).insert(PerformingMovement::default());
            game_info.current_mover = Some(to_move);
        } else {
            // All hath moved, move along
            next_state.set(GameActivity::Conflict);
        }
    }
}

pub fn clear_moves(
    token_query: Query<Entity, With<TokenHasMoved>>,
    mut commands: Commands,
) {
    for token in token_query.iter() {
        commands.entity(token).remove::<TokenHasMoved>();
    }
}

pub fn player_end_movement(
    mut end_event: EventReader<PlayerMovementEnded>,
    mut game_info_and_stuff: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_player: EventWriter<NextPlayerStarted>,
) {
    for _ in end_event.read() {
        if let Some(player) = game_info_and_stuff.current_mover {
            commands.entity(player).remove::<PerformingMovement>();
            game_info_and_stuff.current_mover = None;
            next_player.send(NextPlayerStarted {});
        }
    }
}

pub fn move_tokens_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToAreaCommand>,
    mut pop_query: Query<&mut Population>,
    mut commands: Commands,
    mut player_areas: Query<&mut PlayerAreas>,
    mut recalculate_moves: EventWriter<RecalculatePlayerMoves>,
) {
    for ev in move_events.read() {
        if let Ok(mut from_pop) = pop_query.get_mut(ev.source_area) {
            if let Some(tokens_to_move) = from_pop.remove_tokens_from_area(ev.player, ev.number_of_tokens) {
                if let Ok(mut to_pop) = pop_query.get_mut(ev.target_area) {
                    if let Ok(mut player_area) = player_areas.get_mut(ev.player) {
                        tokens_to_move
                            .iter()
                            .for_each(|token| {
                                commands.entity(*token).insert(TokenHasMoved::default());
                                player_area.remove_token_from_area(ev.source_area, *token);
                                to_pop.add_token_to_area(ev.player, *token);
                                player_area.add_token_to_area(ev.target_area, *token);
                            });
                        recalculate_moves.send(RecalculatePlayerMoves::new(ev.player));
                    }
                }
            }
        }
    }
}
