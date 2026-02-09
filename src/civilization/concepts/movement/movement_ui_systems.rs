use crate::civilization::components::GameArea;
use crate::civilization::concepts::movement::movement_components::PerformingMovement;
use crate::civilization::concepts::movement::movement_events::{
    MoveTokenFromAreaToAreaCommand, PlayerMovementEnded,
};
use crate::civilization::concepts::movement::movement_ui_components::*;
use crate::civilization::game_moves::{AvailableMoves, GameMove, MovementMove};
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to detect when human player has movement options and populate the selection state
pub fn setup_human_movement_options(
    human_players: Query<
        (Entity, &AvailableMoves),
        (
            With<IsHuman>,
            With<PerformingMovement>,
            Added<AvailableMoves>,
        ),
    >,
    mut selection_state: ResMut<MovementSelectionState>,
) {
    for (player_entity, available_moves) in human_players.iter() {
        // Check if there are any movement moves
        let has_movement_moves = available_moves.moves.values().any(|m| {
            matches!(
                m,
                GameMove::Movement(_) | GameMove::AttackArea(_) | GameMove::AttackCity(_)
            )
        });

        if has_movement_moves {
            selection_state.player = Some(player_entity);
            // Don't select a target yet - wait for user interaction
        }
    }
}

/// System to draw arrows from source areas to valid target areas using Gizmos
pub fn draw_movement_arrows(
    mut gizmos: Gizmos,
    human_players: Query<(Entity, &AvailableMoves), (With<IsHuman>, With<PerformingMovement>)>,
    area_transforms: Query<&Transform, With<GameArea>>,
    selection_state: Res<MovementSelectionState>,
) {
    for (_player_entity, available_moves) in human_players.iter() {
        // Group moves by source area
        let mut source_targets: bevy::platform::collections::HashMap<
            Entity,
            Vec<(Entity, bool, bool, usize)>,
        > = bevy::platform::collections::HashMap::new();

        for game_move in available_moves.moves.values() {
            match game_move {
                GameMove::Movement(m) => {
                    source_targets.entry(m.source).or_default().push((
                        m.target,
                        false,
                        false,
                        m.max_tokens,
                    ));
                }
                GameMove::AttackArea(m) => {
                    source_targets.entry(m.source).or_default().push((
                        m.target,
                        true,
                        false,
                        m.max_tokens,
                    ));
                }
                GameMove::AttackCity(m) => {
                    source_targets.entry(m.source).or_default().push((
                        m.target,
                        true,
                        true,
                        m.max_tokens,
                    ));
                }
                _ => {}
            }
        }

        // Draw arrows for each source -> target pair
        for (source, targets) in source_targets.iter() {
            let Ok(source_transform) = area_transforms.get(*source) else {
                continue;
            };
            let source_pos = source_transform.translation.truncate();

            for (target, is_attack, is_city_attack, _max_tokens) in targets {
                // If we have a selection, only show the selected arrow
                if selection_state.has_selection()
                    && (selection_state.source_area != Some(*source)
                        || selection_state.target_area != Some(*target))
                {
                    continue;
                }

                let Ok(target_transform) = area_transforms.get(*target) else {
                    continue;
                };
                let target_pos = target_transform.translation.truncate();

                // Choose color based on move type
                let color = if selection_state.target_area == Some(*target)
                    && selection_state.source_area == Some(*source)
                {
                    Color::srgb(0.2, 1.0, 0.2) // Bright green for selected
                } else if *is_city_attack {
                    Color::srgb(1.0, 0.2, 0.2) // Red for city attack
                } else if *is_attack {
                    Color::srgb(1.0, 0.6, 0.2) // Orange for area attack
                } else {
                    Color::srgb(0.2, 0.6, 1.0) // Blue for normal movement
                };

                // Draw arrow line
                gizmos.arrow_2d(source_pos, target_pos, color);
            }
        }
    }
}

/// System to handle clicking on target areas to select them
pub fn handle_movement_target_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<
        (&Camera, &GlobalTransform),
        With<crate::civilization::components::GameCamera>,
    >,
    human_players: Query<(Entity, &AvailableMoves), (With<IsHuman>, With<PerformingMovement>)>,
    area_query: Query<(Entity, &Transform), With<GameArea>>,
    mut selection_state: ResMut<MovementSelectionState>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // If we already have a selection, don't allow clicking new targets
    if selection_state.has_selection() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    const CLICK_RADIUS: f32 = 30.0;

    for (player_entity, available_moves) in human_players.iter() {
        // Find which target area was clicked
        for game_move in available_moves.moves.values() {
            let movement_move: Option<&MovementMove> = match game_move {
                GameMove::Movement(m) | GameMove::AttackArea(m) | GameMove::AttackCity(m) => {
                    Some(m)
                }
                _ => None,
            };

            if let Some(m) = movement_move {
                if let Ok((_, target_transform)) = area_query.get(m.target) {
                    let target_pos = target_transform.translation.truncate();
                    let distance = world_pos.distance(target_pos);

                    if distance <= CLICK_RADIUS {
                        let is_attack =
                            matches!(game_move, GameMove::AttackArea(_) | GameMove::AttackCity(_));
                        let is_city_attack = matches!(game_move, GameMove::AttackCity(_));

                        selection_state.select_target(
                            player_entity,
                            m.source,
                            m.target,
                            m.max_tokens,
                            is_attack,
                            is_city_attack,
                        );
                        return;
                    }
                }
            }
        }
    }
}

/// System to handle movement control button interactions
pub fn handle_movement_button_clicks(
    interaction_query: Query<
        (&Interaction, &MovementButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut selection_state: ResMut<MovementSelectionState>,
    mut move_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>,
    mut end_movement_writer: MessageWriter<PlayerMovementEnded>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            MovementButtonAction::IncrementTokens => {
                selection_state.increment();
            }
            MovementButtonAction::DecrementTokens => {
                selection_state.decrement();
            }
            MovementButtonAction::ConfirmMove => {
                if let (Some(player), Some(source), Some(target)) = (
                    selection_state.player,
                    selection_state.source_area,
                    selection_state.target_area,
                ) {
                    if selection_state.token_count > 0 {
                        move_writer.write(MoveTokenFromAreaToAreaCommand::new(
                            source,
                            target,
                            selection_state.token_count,
                            player,
                        ));
                        selection_state.clear();
                    }
                }
            }
            MovementButtonAction::CancelMove => {
                selection_state.clear();
            }
            MovementButtonAction::EndMovement => {
                if let Some(player) = selection_state.player {
                    end_movement_writer.write(PlayerMovementEnded::new(player));
                    selection_state.clear();
                }
            }
            MovementButtonAction::SelectTarget {
                source,
                target,
                max_tokens,
                is_attack,
                is_city_attack,
            } => {
                if let Some(player) = selection_state.player {
                    selection_state.select_target(
                        player,
                        *source,
                        *target,
                        *max_tokens,
                        *is_attack,
                        *is_city_attack,
                    );
                }
            }
        }
    }
}

/// System to spawn the movement controls UI when human player is performing movement
pub fn spawn_movement_controls_ui(
    mut commands: Commands,
    human_players: Query<
        Entity,
        (
            With<IsHuman>,
            With<PerformingMovement>,
            Added<AvailableMoves>,
        ),
    >,
    existing_ui: Query<Entity, With<MovementUiRoot>>,
    asset_server: Res<AssetServer>,
) {
    // Don't spawn if UI already exists
    if !existing_ui.is_empty() {
        return;
    }

    for _player in human_players.iter() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        // Spawn the movement controls panel
        commands
            .spawn((
                MovementUiRoot,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ))
            .with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new("Movement"),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Token count display row
                parent
                    .spawn((
                        MovementControlsPanel,
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        // Minus button
                        row.spawn((
                            Button,
                            MovementButtonAction::DecrementTokens,
                            Node {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                        ))
                        .with_child((
                            Text::new("-"),
                            TextFont {
                                font: font.clone(),
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Token count display
                        row.spawn((
                            TokenCountDisplay,
                            Text::new("0"),
                            TextFont {
                                font: font.clone(),
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                min_width: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));

                        // Plus button
                        row.spawn((
                            Button,
                            MovementButtonAction::IncrementTokens,
                            Node {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                        ))
                        .with_child((
                            Text::new("+"),
                            TextFont {
                                font: font.clone(),
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                // Action buttons row
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },))
                    .with_children(|row| {
                        // OK button
                        row.spawn((
                            Button,
                            MovementButtonAction::ConfirmMove,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                        ))
                        .with_child((
                            Text::new("OK"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Cancel button
                        row.spawn((
                            Button,
                            MovementButtonAction::CancelMove,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.3, 0.2)),
                        ))
                        .with_child((
                            Text::new("Cancel"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // End Movement button
                        row.spawn((
                            Button,
                            MovementButtonAction::EndMovement,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.2, 0.4)),
                        ))
                        .with_child((
                            Text::new("End Move"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });

        break; // Only spawn once
    }
}

/// System to update the token count display
pub fn update_token_count_display(
    selection_state: Res<MovementSelectionState>,
    mut text_query: Query<&mut Text, With<TokenCountDisplay>>,
) {
    if !selection_state.is_changed() {
        return;
    }

    for mut text in text_query.iter_mut() {
        if selection_state.has_selection() {
            **text = format!(
                "{} / {}",
                selection_state.token_count, selection_state.max_tokens
            );
        } else {
            **text = "Click target".to_string();
        }
    }
}

/// System to cleanup movement UI when movement phase ends or human player finishes
pub fn cleanup_movement_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<MovementUiRoot>>,
    human_players: Query<Entity, (With<IsHuman>, With<PerformingMovement>)>,
    mut selection_state: ResMut<MovementSelectionState>,
) {
    // If no human player is performing movement, cleanup
    if human_players.is_empty() {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
        selection_state.clear();
    }
}

/// System to cleanup when exiting movement activity
pub fn cleanup_movement_ui_on_exit(
    mut commands: Commands,
    ui_query: Query<Entity, With<MovementUiRoot>>,
    mut selection_state: ResMut<MovementSelectionState>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
    selection_state.clear();
}
