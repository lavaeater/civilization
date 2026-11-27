use crate::civilization::Z_ACTION_UI;
use crate::civilization::components::{GameArea, GameCamera};
use crate::civilization::concepts::movement::movement_components::PerformingMovement;
use crate::civilization::concepts::movement::movement_events::{
    MoveTokenFromAreaToAreaCommand, PlayerMovementEnded, ShipFerryCommand,
};
use crate::civilization::concepts::movement::movement_ui_components::{
    MovementSelectionState, MovementUiRoot, ShipStatusDisplay, SourceAreaDisplay, TokenCountDisplay,
};
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::civilization::{
    CivCardName, PlayerCivilizationCards, PlayerShips, SHIP_CAPACITY, ShipVoyages, ship_range,
};
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{ButtonTheme, LavaTheme, TextStyle, UIBuilder};

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
                GameMove::Movement(m)
                | GameMove::AttackArea(m)
                | GameMove::AttackCity(m)
                | GameMove::ShipFerry(m) => Some(m.source),
                _ => None,
            };
            if let Some(s) = source
                && !source_areas.contains(&s)
            {
                source_areas.push(s);
            }
        }

        if !source_areas.is_empty() {
            selection_state.player = Some(player_entity);
            selection_state.resync_sources(&source_areas);
            // After a leg at sea, stay with the ship.
            if let Some(area) = selection_state.follow_area.take() {
                selection_state.focus(area);
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
            font_size: 12.0,
            text_color: Color::WHITE,
            bg: Color::srgba(0.2, 0.2, 0.2, 0.9),
            bg_hovered: Color::srgba(0.3, 0.3, 0.3, 0.9),
            bg_pressed: Color::srgba(0.4, 0.4, 0.4, 0.9),
            border_color: Color::srgba(0.8, 0.8, 0.8, 0.9),
            border_radius: BorderRadius::ZERO,
            border_width: px(1.0),
            ..default()
        };
        /*
        Get a clone of the theme and modify it for this builder
        */
        let mut theme = ui_theme.clone();
        theme.button = button_theme;

        // Spawn the movement controls panel. `pan_camera_to_current_source`
        // always recenters the camera on the selected source area, so
        // anchoring near screen-center puts this panel right next to the
        // action instead of parked on the bottom edge -- offset up and to
        // one side so it sits beside the (centered) source rather than on
        // top of it. Not full target-avoidance (targets can be in any
        // direction around the source), just a floating-near-the-action
        // default meant to be tuned by eye during play.
        let mut builder = UIBuilder::new(commands, Some(theme));
        builder
            .component::<MovementUiRoot>()
            .z_index(Z_ACTION_UI)
            .absolute_position()
            .top(percent(58.0))
            .left(percent(58.0))
            .flex_column()
            .padding_all(px(10.0))
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.9));

        // Source area navigation row
        builder.add_row(|source_control_row| {
            source_control_row
                .align_items_end()
                .justify_space_between()
                .column_gap(px(8.0))
                .margin_btm(px(8.0));
            source_control_row
                .add_button_observe(
                    "Prev",
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
                            Some(TextStyle {
                                font: Some(font.clone()),
                                font_size: Some(12.0),
                                color: Some(Color::WHITE),
                                justify: Some(Justify::Center),
                                line_break: Some(LineBreak::NoWrap),
                            }),
                        )
                        .text_justify_center()
                        .align_items_center()
                        .width(px(150.0));
                })
                .add_button_observe(
                    "Next",
                    |button| {
                        button
                            .size(px(36.0), px(36.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center);
                    },
                    |_activate: On<Activate>,
                     mut selection_state: ResMut<MovementSelectionState>| {
                        info!("Next source clicked");
                        selection_state.next_source();
                    },
                )
                .add_button_observe(
                    "Skip",
                    |button| {
                        button
                            .size(px(36.0), px(36.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center);
                    },
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
                .justify_space_between()
                .column_gap(px(10.0))
                .margin_btm(px(10.0));
            token_count_row.add_button_observe(
                "-",
                |button| {
                    button
                        .size(px(36.0), px(36.0))
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center);
                },
                |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>| {
                    info!("Minus clicked");
                    selection_state.decrement();
                },
            );

            token_count_row.with_child(|child| {
                child
                    .component::<TokenCountDisplay>()
                    .with_text(
                        "Click target",
                        Some(TextStyle {
                            font: Some(font.clone()),
                            font_size: Some(16.0),
                            color: Some(Color::WHITE),
                            justify: Some(Justify::Center),
                            line_break: Some(LineBreak::NoWrap),
                        }),
                    )
                    .width(px(120.));
            });
            token_count_row.add_button_observe(
                "+",
                |button| {
                    button
                        .size(px(36.0), px(36.0))
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center);
                },
                |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>| {
                    info!("Plus clicked");
                    selection_state.increment();
                },
            );
        });
        // Ship line: what the ship here carries and how far it can still go.
        builder.add_row(|ship_row| {
            ship_row
                .align_items_center()
                .justify_space_between()
                .column_gap(px(8.0))
                .margin_btm(px(6.0));
            ship_row.with_child(|child| {
                child
                    .component::<ShipStatusDisplay>()
                    .with_text(
                        "No ship here",
                        Some(TextStyle {
                            font: Some(font.clone()),
                            font_size: Some(12.0),
                            color: Some(Color::srgb(0.6, 0.9, 0.9)),
                            justify: Some(Justify::Left),
                            line_break: Some(LineBreak::NoWrap),
                        }),
                    )
                    .width(px(300.0));
            });
        });
        // Action buttons row - OK and End Movement use markers for global observers (need MessageWriter)
        builder.add_row(|action_row| {
            action_row.column_gap(px(10.))
                .justify_space_between()
                .align_items_end();
            action_row
                .add_button_observe(
                    "OK",
                    |button| {
                        button
                            .size(px(36.0), px(36.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center);
                    },
                    |_activate: On<Activate>,
                        mut selection_state: ResMut<MovementSelectionState>,
                     mut move_writer: MessageWriter<MoveTokenFromAreaToAreaCommand>,
                     mut ferry_writer: MessageWriter<ShipFerryCommand>| {
                        info!("OK clicked");
                        if selection_state.by_ship
                            && let (Some(player), Some(source), Some(target)) = (
                                selection_state.player,
                                selection_state.source_area,
                                selection_state.target_area,
                            )
                        {
                            let (drop, load) = selection_state.ferry_split();
                            ferry_writer.write(ShipFerryCommand::leg(
                                source, target, drop, load, player,
                            ));
                            selection_state.follow_area = Some(target);
                            selection_state.clear_target();
                        } else if let (Some(player), Some(source), Some(target)) = (
                            selection_state.player,
                            selection_state.source_area,
                            selection_state.target_area,
                        ) && selection_state.token_count > 0 {
                            move_writer.write(MoveTokenFromAreaToAreaCommand::new(
                                source,
                                target,
                                selection_state.token_count,
                                player,
                            ));
                            selection_state.clear_target();
                        }
                    },
                ).add_button_observe(
                "Cancel",
                |button| {
                    button
                        .size(px(36.0), px(36.0))
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center);
                },
                |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>| {
                    info!("Cancel clicked");
                    selection_state.clear_target();
                },
            ).add_button_observe(
                "End",
                |button| {
                    button
                        .size(px(36.0), px(36.0))
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center);
                },
                |_activate: On<Activate>, mut selection_state: ResMut<MovementSelectionState>, mut end_movement_writer: MessageWriter<PlayerMovementEnded>| {
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

    for mut text in &mut text_query {
        if selection_state.has_selection() && selection_state.by_ship {
            **text = format!(
                "Aboard {} / {}",
                selection_state.token_count, selection_state.max_tokens
            );
        } else if selection_state.has_selection() {
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

    for mut text in &mut text_query {
        if selection_state.all_skipped() {
            **text = "All sources skipped".to_string();
        } else if let Some(source) = selection_state.current_source() {
            let area_name = area_names
                .get(source)
                .map_or("?", bevy::prelude::Name::as_str);
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

/// The ship line: for the ship at the current source how many are aboard and
/// how far it can still sail -- and, once a sea move is picked, who steps
/// off and who steps on.
pub fn update_ship_status_display(
    mut selection_state: ResMut<MovementSelectionState>,
    voyages: Res<ShipVoyages>,
    players: Query<(&PlayerShips, Option<&PlayerCivilizationCards>)>,
    mut status_text: Query<&mut Text, With<ShipStatusDisplay>>,
) {
    let ships_info = selection_state
        .player
        .zip(selection_state.current_source())
        .and_then(|(player, source)| {
            let (ships, cards) = players.get(player).ok()?;
            let here = ships.ships_in_area(source);
            if here.is_empty() {
                return None;
            }
            let range = ship_range(cards.is_some_and(|c| c.owns(&CivCardName::ClothMaking)));
            let ship = voyages.pick_ship(here, range, 1).unwrap_or(here[0]);
            Some((
                voyages.get(ship).cargo.len(),
                voyages.areas_left(ship, range),
            ))
        });
    let cargo = ships_info.map_or(0, |(cargo, _)| cargo);
    if selection_state.ship_cargo != cargo {
        selection_state.ship_cargo = cargo;
    }
    if !selection_state.is_changed() && !voyages.is_changed() {
        return;
    }
    let status = match ships_info {
        None => "No ship here".to_string(),
        Some((cargo, left)) if selection_state.by_ship && selection_state.has_selection() => {
            let (off, on) = selection_state.ferry_split();
            let change = match (off, on) {
                (0, 0) => "nobody changes".to_string(),
                (off, 0) => format!("{off} step off here"),
                (0, on) => format!("{on} step on"),
                (off, on) => format!("{off} off, {on} on"),
            };
            format!("Ship: {cargo} aboard, {change}; {left} sea area(s) left")
        }
        Some((cargo, left)) => format!(
            "Ship: {cargo}/{SHIP_CAPACITY} aboard, {left} sea area(s) left. Shift-click sails"
        ),
    };
    for mut text in &mut status_text {
        text.0.clone_from(&status);
    }
}
