use bevy::prelude::*;

use crate::civilization::components::GameArea;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::*;
use crate::stupid_ai::IsHuman;

/// Spawn the calamity city-selection panel when a human player gets
/// `AwaitingHumanCalamitySelection` inserted.
pub fn spawn_calamity_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingHumanCalamitySelection>)>,
    existing_ui: Query<Entity, With<CalamitySelectionUiRoot>>,
    calamity_selection: Res<CalamitySelectionState>,
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
            CalamitySelectionUiRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.05, 0.15, 0.93)),
        ))
        .with_children(|parent| {
            // Title row: "CIVIL DISORDER — Select 3 cities to reduce"
            parent.spawn((
                CalamitySelectionTitleText,
                Text::new(format!(
                    "{} — Select {} cit{}",
                    calamity_selection.calamity_name,
                    calamity_selection.required_count,
                    if calamity_selection.required_count == 1 { "y" } else { "ies" }
                )),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(1.0, 0.7, 0.2)),
            ));

            // City navigation row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    // Prev button
                    row.spawn((
                        Button,
                        CalamitySelectionButtonAction::PrevCity,
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

                    // City name
                    row.spawn((
                        CalamitySelectionCityNameText,
                        Text::new("?"),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(180.0), ..default() },
                    ));

                    // Next button
                    row.spawn((
                        Button,
                        CalamitySelectionButtonAction::NextCity,
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

            // Toggle + progress row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    // Toggle select/deselect button
                    row.spawn((
                        Button,
                        CalamitySelectionButtonAction::ToggleCity,
                        CalamitySelectionToggleButton,
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.4, 0.3)),
                    ))
                    .with_child((
                        Text::new("Select"),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    // Progress display
                    row.spawn((
                        CalamitySelectionProgressText,
                        Text::new(format!("0 / {}", calamity_selection.required_count)),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });

            // Confirm button (greyed out until selection complete)
            parent
                .spawn((
                    Button,
                    CalamitySelectionButtonAction::Confirm,
                    CalamitySelectionConfirmButton,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_child((
                    Text::new("Confirm"),
                    TextFont { font: font.clone(), font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
        });
}

/// Update city name text, toggle button label, progress, and confirm button colour.
pub fn update_calamity_selection_ui(
    calamity_selection: Res<CalamitySelectionState>,
    area_names: Query<&Name, With<GameArea>>,
    mut title_text: Query<&mut Text, (With<CalamitySelectionTitleText>, Without<CalamitySelectionCityNameText>, Without<CalamitySelectionProgressText>)>,
    mut city_name_text: Query<&mut Text, (With<CalamitySelectionCityNameText>, Without<CalamitySelectionTitleText>, Without<CalamitySelectionProgressText>)>,
    mut progress_text: Query<&mut Text, (With<CalamitySelectionProgressText>, Without<CalamitySelectionTitleText>, Without<CalamitySelectionCityNameText>)>,
    mut toggle_button: Query<(&mut BackgroundColor, &Children), With<CalamitySelectionToggleButton>>,
    mut confirm_button: Query<(&mut BackgroundColor, &Children), (With<CalamitySelectionConfirmButton>, Without<CalamitySelectionToggleButton>)>,
    mut child_texts: Query<&mut Text, Without<CalamitySelectionTitleText>>,
) {
    if !calamity_selection.is_changed() {
        return;
    }

    // Update title
    if let Ok(mut text) = title_text.single_mut() {
        **text = format!(
            "{} — Select {} cit{}",
            calamity_selection.calamity_name,
            calamity_selection.required_count,
            if calamity_selection.required_count == 1 { "y" } else { "ies" }
        );
    }

    // Update city name
    if let Ok(mut text) = city_name_text.single_mut() {
        if let Some(city) = calamity_selection.current_city() {
            let name = area_names.get(city).map(|n| n.as_str()).unwrap_or("?");
            let selected_marker = if calamity_selection.is_current_selected() { " [X]" } else { "" };
            **text = format!(
                "{}{} ({}/{})",
                name,
                selected_marker,
                calamity_selection.current_index + 1,
                calamity_selection.available_cities.len()
            );
        } else {
            **text = "No cities available".to_string();
        }
    }

    // Update progress
    if let Ok(mut text) = progress_text.single_mut() {
        **text = format!(
            "{} / {} selected",
            calamity_selection.selected_cities.len(),
            calamity_selection.required_count
        );
    }

    // Update toggle button label and color
    let is_selected = calamity_selection.is_current_selected();
    if let Ok((mut bg, children)) = toggle_button.single_mut() {
        *bg = if is_selected {
            BackgroundColor(Color::srgb(0.5, 0.2, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.2, 0.4, 0.2))
        };
        for child in children.iter() {
            if let Ok(mut text) = child_texts.get_mut(child) {
                **text = if is_selected { "Deselect".to_string() } else { "Select".to_string() };
            }
        }
    }

    // Update confirm button color based on selection completion
    let complete = calamity_selection.selection_complete();
    if let Ok((mut bg, _)) = confirm_button.single_mut() {
        *bg = if complete {
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2))
        };
    }
}

/// Handle button clicks in the calamity selection UI.
pub fn handle_calamity_selection_buttons(
    interaction_query: Query<
        (&Interaction, &CalamitySelectionButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut calamity_selection: ResMut<CalamitySelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            CalamitySelectionButtonAction::PrevCity => {
                calamity_selection.prev();
            }
            CalamitySelectionButtonAction::NextCity => {
                calamity_selection.next();
            }
            CalamitySelectionButtonAction::ToggleCity => {
                calamity_selection.toggle_current();
            }
            CalamitySelectionButtonAction::Confirm => {
                if calamity_selection.selection_complete() {
                    // Signal the advance system by removing the waiting marker
                    if let Ok(player) = human_waiting.single() {
                        info!("[CALAMITY UI] Human confirmed {} city selection(s)",
                            calamity_selection.selected_cities.len());
                        commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                    }
                }
            }
        }
    }
}

/// Despawn the calamity selection UI when no human player is waiting.
pub fn cleanup_calamity_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<CalamitySelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Force-cleanup the UI when exiting the ResolveCalamities activity.
pub fn cleanup_calamity_selection_ui_on_exit(
    mut commands: Commands,
    ui_root: Query<Entity, With<CalamitySelectionUiRoot>>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for entity in ui_root.iter() {
        commands.entity(entity).despawn();
    }
    calamity_selection.clear();
}
