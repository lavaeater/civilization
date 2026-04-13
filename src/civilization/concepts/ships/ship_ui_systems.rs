use bevy::prelude::*;

use crate::civilization::components::GameArea;
use crate::civilization::concepts::ships::ship_ui_components::*;
use crate::stupid_ai::IsHuman;

/// Spawn the ship construction panel when a human player has `AwaitingShipPlacement`.
pub fn spawn_ship_construction_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingShipPlacement>)>,
    existing_ui: Query<Entity, With<ShipConstructionUiRoot>>,
    ship_state: Res<ShipConstructionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !existing_ui.is_empty() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            ShipConstructionUiRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.1, 0.2, 0.93)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Ship Construction"),
                TextFont { font: font.clone(), font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.4, 0.8, 1.0)),
            ));

            // Count row: "─  N ships  +"
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    // Decrement button
                    row.spawn((
                        Button,
                        ShipConstructionButtonAction::Decrement,
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    ))
                    .with_child((
                        Text::new("−"),
                        TextFont { font: font.clone(), font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    // Count display
                    row.spawn((
                        ShipCountText,
                        Text::new(format!(
                            "0 ships  (max {})",
                            ship_state.max_buildable
                        )),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node { min_width: Val::Px(160.0), ..default() },
                    ));

                    // Increment button
                    row.spawn((
                        Button,
                        ShipConstructionButtonAction::Increment,
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    ))
                    .with_child((
                        Text::new("+"),
                        TextFont { font: font.clone(), font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // Slot navigation row (only shown when ships_to_build > 0)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        ShipConstructionButtonAction::PrevSlot,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.25, 0.4)),
                    ))
                    .with_child((
                        Text::new("◄"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    row.spawn((
                        ShipSlotText,
                        Text::new("—"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 1.0)),
                        Node { min_width: Val::Px(80.0), ..default() },
                    ));

                    row.spawn((
                        Button,
                        ShipConstructionButtonAction::NextSlot,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.25, 0.4)),
                    ))
                    .with_child((
                        Text::new("►"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // Area navigation row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        ShipConstructionButtonAction::PrevArea,
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    ))
                    .with_child((
                        Text::new("<"),
                        TextFont { font: font.clone(), font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    row.spawn((
                        ShipAreaText,
                        Text::new("no area"),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(180.0), ..default() },
                    ));

                    row.spawn((
                        Button,
                        ShipConstructionButtonAction::NextArea,
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    ))
                    .with_child((
                        Text::new(">"),
                        TextFont { font: font.clone(), font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // Confirm button
            parent
                .spawn((
                    Button,
                    ShipConstructionButtonAction::Confirm,
                    ShipConfirmButton,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                ))
                .with_child((
                    Text::new("Confirm"),
                    TextFont { font: font.clone(), font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));
        });
}

/// Keep the panel text in sync with `ShipConstructionState`.
pub fn update_ship_construction_ui(
    ship_state: Res<ShipConstructionState>,
    area_names: Query<&Name, With<GameArea>>,
    mut count_text: Query<
        &mut Text,
        (With<ShipCountText>, Without<ShipSlotText>, Without<ShipAreaText>),
    >,
    mut slot_text: Query<
        &mut Text,
        (With<ShipSlotText>, Without<ShipCountText>, Without<ShipAreaText>),
    >,
    mut area_text: Query<
        &mut Text,
        (With<ShipAreaText>, Without<ShipCountText>, Without<ShipSlotText>),
    >,
) {
    if !ship_state.is_changed() {
        return;
    }

    if let Ok(mut t) = count_text.single_mut() {
        **t = format!(
            "{} ship{}  (max {})",
            ship_state.ships_to_build,
            if ship_state.ships_to_build == 1 { "" } else { "s" },
            ship_state.max_buildable
        );
    }

    if let Ok(mut t) = slot_text.single_mut() {
        if ship_state.ships_to_build == 0 {
            **t = "—".to_string();
        } else {
            **t = format!(
                "Ship {}/{}",
                ship_state.current_slot + 1,
                ship_state.ships_to_build
            );
        }
    }

    if let Ok(mut t) = area_text.single_mut() {
        if ship_state.ships_to_build == 0 {
            **t = "—".to_string();
        } else if let Some(area) = ship_state.current_area() {
            let name = area_names.get(area).map(|n| n.as_str()).unwrap_or("?");
            **t = format!(
                "{} ({}/{})",
                name,
                ship_state.area_cursor + 1,
                ship_state.available_areas.len()
            );
        } else {
            **t = "No areas available".to_string();
        }
    }
}

/// Handle button presses in the ship construction panel.
pub fn handle_ship_construction_buttons(
    interaction_query: Query<
        (&Interaction, &ShipConstructionButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut ship_state: ResMut<ShipConstructionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingShipPlacement>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            ShipConstructionButtonAction::Increment => {
                ship_state.increment();
            }
            ShipConstructionButtonAction::Decrement => {
                ship_state.decrement();
            }
            ShipConstructionButtonAction::PrevArea => {
                ship_state.prev_area();
            }
            ShipConstructionButtonAction::NextArea => {
                ship_state.next_area();
            }
            ShipConstructionButtonAction::PrevSlot => {
                ship_state.prev_slot();
            }
            ShipConstructionButtonAction::NextSlot => {
                ship_state.next_slot();
            }
            ShipConstructionButtonAction::Confirm => {
                if let Ok(player) = human_waiting.single() {
                    info!(
                        "[SHIPS UI] Human confirmed: build {} ships",
                        ship_state.ships_to_build
                    );
                    commands.entity(player).remove::<AwaitingShipPlacement>();
                }
            }
        }
    }
}

/// Despawn the panel when `AwaitingShipPlacement` is removed from the human.
pub fn despawn_ship_construction_ui(
    mut removed: RemovedComponents<AwaitingShipPlacement>,
    ui_root: Query<Entity, With<ShipConstructionUiRoot>>,
    mut commands: Commands,
) {
    for _ in removed.read() {
        for root in ui_root.iter() {
            commands.entity(root).despawn();
        }
    }
}
