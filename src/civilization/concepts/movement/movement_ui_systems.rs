use crate::civilization::components::{GameArea, GameCamera};
use crate::civilization::concepts::movement::movement_components::PerformingMovement;
use crate::civilization::concepts::movement::movement_events::{
    MoveTokenFromAreaToAreaCommand, PlayerMovementEnded,
};
use crate::civilization::concepts::movement::movement_ui_components::{
    MovementSelectionState, MovementUiRoot, SourceAreaDisplay, TokenCountDisplay,
};
use crate::civilization::game_moves::{AvailableMoves, GameMove, MovementMove};
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::PrimaryWindow;
use lava_ui_builder::{ButtonTheme, LavaTheme, UIBuilder};

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
        // Collect unique source areas from movement moves
        let mut source_areas: Vec<Entity> = Vec::new();
        for game_move in available_moves.moves.values() {
            let source = match game_move {
                GameMove::Movement(m) | GameMove::AttackArea(m) | GameMove::AttackCity(m) => {
                    Some(m.source)
                }
                _ => None,
            };
            if let Some(s) = source {
                if !source_areas.contains(&s) {
                    source_areas.push(s);
                }
            }
        }

        if !source_areas.is_empty() {
            selection_state.player = Some(player_entity);
            selection_state.source_areas = source_areas;
            selection_state.current_source_index = 0;
            selection_state.next_source(); //Advance to first unskipped source
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

        // Only draw arrows from the currently focused source area
        let focused_source = selection_state.current_source();

        for (source, targets) in source_targets.iter() {
            // Skip sources that aren't the focused one
            if let Some(focused) = focused_source {
                if *source != focused {
                    continue;
                }
            }

            let Ok(source_transform) = area_transforms.get(*source) else {
                continue;
            };
            let source_pos = source_transform.translation.truncate();

            for (target, is_attack, is_city_attack, _max_tokens) in targets {
                // If we have a target selection, only show the selected arrow
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
                gizmos
                    .arrow_2d(source_pos, target_pos, color)
                    .with_double_end();
            }
        }
    }
}

/// System to handle clicking on target areas to select them
pub fn handle_movement_target_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    human_players: Query<(Entity, &AvailableMoves), (With<IsHuman>, With<PerformingMovement>)>,
    area_query: Query<(Entity, &Transform), With<GameArea>>,
    mut selection_state: ResMut<MovementSelectionState>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
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

    let focused_source = selection_state.current_source();

    for (player_entity, available_moves) in human_players.iter() {
        // Find which target area was clicked (only from focused source)
        for game_move in available_moves.moves.values() {
            let movement_move: Option<&MovementMove> = match game_move {
                GameMove::Movement(m) | GameMove::AttackArea(m) | GameMove::AttackCity(m) => {
                    Some(m)
                }
                _ => None,
            };

            if let Some(m) = movement_move {
                // Only allow clicking targets from the focused source
                if let Some(focused) = focused_source {
                    if m.source != focused {
                        continue;
                    }
                }

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

/// System to spawn the movement controls UI when human player is performing movement
pub fn spawn_movement_controls_ui(
    commands: Commands,
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
    ui_theme: Res<LavaTheme>,
) {
    // Don't spawn if UI already exists
    if !existing_ui.is_empty() {
        return;
    }

    if let Some(player) = human_players.iter().next() {
        debug!(
            "Spawning movement controls UI for human player {:?}",
            player
        );
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let button_theme = ButtonTheme {
            font: font.clone(),
            font_size: 20.0,
            text_color: Color::WHITE,
            bg: Color::srgba(0.2, 0.2, 0.2, 0.9),
            bg_hovered: Color::srgba(0.3, 0.3, 0.3, 0.9),
            bg_pressed: Color::srgba(0.4, 0.4, 0.4, 0.9),
            ..default()
        };
        /*
        Get a clone of the theme and modify it for this builder
        */
        let mut theme = ui_theme.clone();
        theme.button = button_theme;

        // Spawn the movement controls panel
        let mut builder = UIBuilder::new(commands, Some(theme));
        builder
            .component::<MovementUiRoot>()
            .absolute_position()
            .bottom(px(20.0))
            .left(percent(50.0))
            .flex_column()
            .padding_all(px(10.0))
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.9));

        // Source area navigation row
        builder.add_row(|source_control_row| {
            source_control_row
                .align_items_center()
                .column_gap(px(8.0))
                .margin_btm(px(8.0));
            source_control_row
                .add_button_observe(
                    "Previous Source",
                    |button| {
                        button
                            .size(px(36.0), px(36.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center);
                    },
                    |_activate: On<Activate>,
                     mut selection_state: ResMut<MovementSelectionState>| {
                        info!("Previous source clicked");
                        selection_state.prev_source();
                    },
                )
                .with_child(|area_name_display| {
                    area_name_display
                        .component::<SourceAreaDisplay>()
                        .with_text(
                            "Source: ?",
                            Some(font.clone()),
                            Some(20.0),
                            Some(Color::WHITE),
                            Some(Justify::Center),
                            Some(LineBreak::NoWrap),
                        )
                        .width(px(200.));
                })
                .feathers_button(
                    "Next Source",
                    |_activate: On<Activate>,
                     mut selection_state: ResMut<MovementSelectionState>| {
                        info!("Next source clicked");
                        selection_state.next_source();
                    },
                )
                .feathers_button(
                    "Skip Source",
                    |_activate: On<Activate>,
                     mut selection_state: ResMut<MovementSelectionState>| {
                        info!("Skip source clicked");
                        selection_state.skip_current_source();
                    },
                );
        });
        // Token count display row
        builder.add_row(|token_count_row| {
            token_count_row
                .align_items_center()
                .column_gap(px(10.0))
                .margin_btm(px(10.0));
            token_count_row.add_column(|c| {
                c.feathers_button("-", |_activate: On<Activate>| {
                    info!("Minus clicked");
                    // , mut selection_state: ResMut<MovementSelectionState>
                    // selection_state.decrement();
                })
                .border_all(px(5.0), Color::srgba(0.1, 0.1, 0.1, 0.9));
            });

            token_count_row.with_child(|child| {
                child
                    .component::<TokenCountDisplay>()
                    .with_text(
                        "Click target",
                        Some(font.clone()),
                        Some(24.0),
                        Some(Color::WHITE),
                        Some(Justify::Center),
                        Some(LineBreak::NoWrap),
                    )
                    .width(px(120.));
            });
            token_count_row
                .feathers_button("+", |_activate: On<Activate>| {
                    info!("Plus clicked");
                    // , mut selection_state: ResMut<MovementSelectionState>                selection_state.increment();
                })
                .border_all(px(5.0), Color::srgba(0.1, 0.1, 0.1, 0.9));
        });
        // Action buttons row - OK and End Movement use markers for global observers (need MessageWriter)
        builder.add_row(|action_row| {
            action_row.column_gap(px(10.));
            action_row
                .feathers_button("OK", |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>, mut move_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>| {
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
                            selection_state.clear_preserving_skips();
                        }
                    }                })
                .feathers_button("Cancel", |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>| {
                    info!("Cancel clicked");
                    selection_state.clear_target();
                })
                .feathers_button("End Movement", |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>, mut end_movement_writer: MessageWriter<PlayerMovementEnded>| {
                    info!("End Movement clicked");
                    if let Some(player) = selection_state.player {
                        end_movement_writer.write(PlayerMovementEnded::new(player));
                        selection_state.clear();
                    }
                });
        });
        builder.build();
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

/// System to update the source area display text
pub fn update_source_area_display(
    selection_state: Res<MovementSelectionState>,
    mut text_query: Query<&mut Text, With<SourceAreaDisplay>>,
    area_names: Query<&Name, With<GameArea>>,
) {
    if !selection_state.is_changed() {
        return;
    }

    let unskipped_count = selection_state
        .source_areas
        .iter()
        .filter(|s| !selection_state.skipped_sources.contains(*s))
        .count();

    for mut text in text_query.iter_mut() {
        if selection_state.all_skipped() {
            **text = "All sources skipped".to_string();
        } else if let Some(source) = selection_state.current_source() {
            let area_name = area_names.get(source).map(|n| n.as_str()).unwrap_or("?");
            let skipped = if selection_state.is_current_skipped() {
                " [SKIPPED]"
            } else {
                ""
            };
            **text = format!(
                "{}{} ({}/{} active)",
                area_name,
                skipped,
                unskipped_count,
                selection_state.source_areas.len()
            );
        } else {
            **text = "No sources".to_string();
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

/// System to pan the camera to the current source area when it changes
pub fn pan_camera_to_current_source(
    selection_state: Res<MovementSelectionState>,
    area_transforms: Query<&Transform, With<GameArea>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<GameArea>)>,
) {
    if !selection_state.is_changed() {
        return;
    }

    let Some(current_source) = selection_state.current_source() else {
        return;
    };

    let Ok(area_transform) = area_transforms.get(current_source) else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = area_transform.translation.x;
    camera_transform.translation.y = area_transform.translation.y;
}
