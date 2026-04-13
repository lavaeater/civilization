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
    cw_ui_root: Query<Entity, With<CivilWarSelectionUiRoot>>,
    mono_ui_root: Query<Entity, With<MonotheismSelectionUiRoot>>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
    mut cw_selection: ResMut<CivilWarSelectionState>,
    mut mono_selection: ResMut<MonotheismSelectionState>,
) {
    for entity in ui_root.iter() {
        commands.entity(entity).despawn();
    }
    for entity in cw_ui_root.iter() {
        commands.entity(entity).despawn();
    }
    for entity in mono_ui_root.iter() {
        commands.entity(entity).despawn();
    }
    calamity_selection.clear();
    cw_selection.clear();
    mono_selection.clear();
}

// ── Civil War selection UI ────────────────────────────────────────────────────

/// Spawn the Civil War selection panel when a human has `AwaitingHumanCalamitySelection`
/// and the `CivilWarSelectionState` resource has an acting player set.
pub fn spawn_civil_war_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingHumanCalamitySelection>)>,
    existing_ui: Query<Entity, With<CivilWarSelectionUiRoot>>,
    cw_selection: Res<CivilWarSelectionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // Only spawn if this is a civil war selection (acting_player set by CW system)
    if !existing_ui.is_empty() || cw_selection.acting_player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let role_label = match cw_selection.role {
        CivilWarUiRole::Victim => "Civil War — Victim Selection",
        CivilWarUiRole::Beneficiary => "Civil War — Beneficiary Selection",
    };
    let hint = match cw_selection.role {
        CivilWarUiRole::Victim => format!("Select at least {} pts to yield", cw_selection.target_points),
        CivilWarUiRole::Beneficiary => format!("Take up to {} pts from the pool", cw_selection.target_points),
    };

    commands
        .spawn((
            CivilWarSelectionUiRoot,
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
            BackgroundColor(Color::srgba(0.1, 0.05, 0.05, 0.93)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                CivilWarTitleText,
                Text::new(role_label),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(1.0, 0.5, 0.3)),
            ));

            // Hint
            parent.spawn((
                Text::new(hint),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Points display
            parent.spawn((
                CivilWarPointsText,
                Text::new("Points: 0 / ?"),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));

            // Tab row: Tokens | Cities
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        CivilWarButtonAction::TokensTab,
                        Node {
                            width: Val::Px(90.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.4, 0.2, 0.1)),
                    ))
                    .with_child((
                        Text::new("Tokens"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Button,
                        CivilWarButtonAction::CitiesTab,
                        Node {
                            width: Val::Px(90.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.4)),
                    ))
                    .with_child((
                        Text::new("Cities"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // Token row: [−] N tokens [+] (of M)
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
                        CivilWarButtonAction::DecrementTokens,
                        Node {
                            width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                    ))
                    .with_child((Text::new("−"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        CivilWarTokenCountText,
                        Text::new(format!("0 tokens  (of {})", cw_selection.total_available_tokens)),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node { min_width: Val::Px(160.0), ..default() },
                    ));

                    row.spawn((
                        Button,
                        CivilWarButtonAction::IncrementTokens,
                        Node {
                            width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.3, 0.1)),
                    ))
                    .with_child((Text::new("+"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                });

            // City navigation row (only shown when cities exist)
            if !cw_selection.available_cities.is_empty() {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Button, CivilWarButtonAction::PrevCity,
                            Node { width: Val::Px(28.0), height: Val::Px(28.0),
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                        )).with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                        row.spawn((
                            CivilWarCityNameText,
                            Text::new("City: ?"),
                            TextFont { font: font.clone(), font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 1.0, 0.7)),
                            Node { min_width: Val::Px(160.0), ..default() },
                        ));

                        row.spawn((
                            Button, CivilWarButtonAction::NextCity,
                            Node { width: Val::Px(28.0), height: Val::Px(28.0),
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                        )).with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                        row.spawn((
                            Button, CivilWarButtonAction::ToggleCity,
                            CivilWarToggleCityButton,
                            Node { width: Val::Px(80.0), height: Val::Px(28.0),
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.2)),
                        )).with_child((Text::new("Select"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));
                    });
            }

            // Confirm button
            parent.spawn((
                Button,
                CivilWarButtonAction::Confirm,
                CivilWarConfirmButton,
                Node {
                    width: Val::Px(160.0), height: Val::Px(40.0),
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
            ))
            .with_child((Text::new("Confirm"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
        });
}

/// Update Civil War UI text each frame the state changes.
pub fn update_civil_war_selection_ui(
    cw_selection: Res<CivilWarSelectionState>,
    area_names: Query<&Name, With<GameArea>>,
    mut points_text: Query<&mut Text, (With<CivilWarPointsText>, Without<CivilWarTokenCountText>, Without<CivilWarCityNameText>)>,
    mut token_text: Query<&mut Text, (With<CivilWarTokenCountText>, Without<CivilWarPointsText>, Without<CivilWarCityNameText>)>,
    mut city_text: Query<&mut Text, (With<CivilWarCityNameText>, Without<CivilWarPointsText>, Without<CivilWarTokenCountText>)>,
    mut toggle_button: Query<(&mut BackgroundColor, &Children), With<CivilWarToggleCityButton>>,
    mut confirm_button: Query<(&mut BackgroundColor, &Children), (With<CivilWarConfirmButton>, Without<CivilWarToggleCityButton>)>,
    mut child_texts: Query<&mut Text, (Without<CivilWarPointsText>, Without<CivilWarTokenCountText>, Without<CivilWarCityNameText>)>,
) {
    if !cw_selection.is_changed() { return; }

    if let Ok(mut text) = points_text.single_mut() {
        let pts = cw_selection.current_points();
        let target = cw_selection.target_points;
        let label = match cw_selection.role {
            CivilWarUiRole::Victim => format!("Points: {} / {} (need ≥{})", pts, target, target),
            CivilWarUiRole::Beneficiary => format!("Points: {} / {} (take up to {})", pts, target, target),
        };
        **text = label;
    }

    if let Ok(mut text) = token_text.single_mut() {
        **text = format!(
            "{} tokens  (of {})",
            cw_selection.selected_token_count,
            cw_selection.total_available_tokens
        );
    }

    if let Ok(mut text) = city_text.single_mut() {
        if let Some(city) = cw_selection.current_city() {
            let name = area_names.get(city).map(|n| n.as_str()).unwrap_or("?");
            let sel = if cw_selection.is_current_city_selected() { " [✓]" } else { "" };
            **text = format!(
                "{}{} ({}/{})",
                name, sel,
                cw_selection.current_city_index + 1,
                cw_selection.available_cities.len()
            );
        }
    }

    let is_selected = cw_selection.is_current_city_selected();
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

    let valid = cw_selection.selection_valid();
    if let Ok((mut bg, _)) = confirm_button.single_mut() {
        *bg = if valid {
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25))
        };
    }
}

/// Handle Civil War UI button presses.
pub fn handle_civil_war_selection_buttons(
    interaction_query: Query<
        (&Interaction, &CivilWarButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut cw_selection: ResMut<CivilWarSelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed { continue; }
        match action {
            CivilWarButtonAction::TokensTab => { cw_selection.showing_cities = false; }
            CivilWarButtonAction::CitiesTab => { cw_selection.showing_cities = true; }
            CivilWarButtonAction::IncrementTokens => { cw_selection.increment_tokens(); }
            CivilWarButtonAction::DecrementTokens => { cw_selection.decrement_tokens(); }
            CivilWarButtonAction::PrevCity => { cw_selection.prev_city(); }
            CivilWarButtonAction::NextCity => { cw_selection.next_city(); }
            CivilWarButtonAction::ToggleCity => { cw_selection.toggle_current_city(); }
            CivilWarButtonAction::Confirm => {
                if cw_selection.selection_valid() {
                    if let Ok(player) = human_waiting.single() {
                        info!("[CIVIL WAR UI] Human confirmed: {} tokens, {} cities",
                            cw_selection.selected_token_count, cw_selection.selected_cities.len());
                        commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                    }
                }
            }
        }
    }
}

/// Despawn Civil War UI when no human player is waiting and acting_player is cleared.
pub fn cleanup_civil_war_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<CivilWarSelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
    cw_selection: Res<CivilWarSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && cw_selection.acting_player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Monotheism target-selection UI ───────────────────────────────────────────

/// Spawn the Monotheism panel when the human holder gets `AwaitingMonotheismSelection`.
pub fn spawn_monotheism_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingMonotheismSelection>)>,
    existing_ui: Query<Entity, With<MonotheismSelectionUiRoot>>,
    mono_state: Res<MonotheismSelectionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !existing_ui.is_empty() || mono_state.player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let max = mono_state.candidates.len().min(2);

    commands
        .spawn((
            MonotheismSelectionUiRoot,
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
            BackgroundColor(Color::srgba(0.05, 0.1, 0.15, 0.93)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Monotheism"),
                TextFont { font: font.clone(), font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.4, 0.8, 1.0)),
            ));
            parent.spawn((
                Text::new(format!("Select up to {} enemy token{} to eliminate", max,
                    if max == 1 { "" } else { "s" })),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Target navigation row
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
                        MonotheismButtonAction::Prev,
                        Node {
                            width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    ))
                    .with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        MonotheismTargetText,
                        Text::new("?"),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(200.0), ..default() },
                    ));

                    row.spawn((
                        Button,
                        MonotheismButtonAction::Next,
                        Node {
                            width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    ))
                    .with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
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
                    row.spawn((
                        Button,
                        MonotheismButtonAction::Toggle,
                        MonotheismToggleButton,
                        Node {
                            width: Val::Px(110.0), height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.4, 0.2)),
                    ))
                    .with_child((Text::new("Eliminate"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        MonotheismProgressText,
                        Text::new(format!("0 / {}", max)),
                        TextFont { font: font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });

            // Confirm button (always enabled — 0 is valid: "do nothing")
            parent.spawn((
                Button,
                MonotheismButtonAction::Confirm,
                MonotheismConfirmButton,
                Node {
                    width: Val::Px(160.0), height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
            ))
            .with_child((Text::new("Confirm"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
        });
}

/// Keep the Monotheism panel text in sync with `MonotheismSelectionState`.
pub fn update_monotheism_selection_ui(
    mono_state: Res<MonotheismSelectionState>,
    area_names: Query<&Name, With<GameArea>>,
    mut target_text: Query<
        &mut Text,
        (With<MonotheismTargetText>, Without<MonotheismProgressText>),
    >,
    mut progress_text: Query<
        &mut Text,
        (With<MonotheismProgressText>, Without<MonotheismTargetText>),
    >,
    mut toggle_button: Query<(&mut BackgroundColor, &Children), With<MonotheismToggleButton>>,
    mut child_texts: Query<&mut Text, (Without<MonotheismTargetText>, Without<MonotheismProgressText>)>,
) {
    if !mono_state.is_changed() {
        return;
    }

    if let Ok(mut t) = target_text.single_mut() {
        if let Some((_, area)) = mono_state.current_candidate() {
            let area_name = area_names.get(area).map(|n| n.as_str()).unwrap_or("?");
            let sel = if mono_state.is_current_selected() { " [✓]" } else { "" };
            **t = format!(
                "{}{} ({}/{})",
                area_name, sel,
                mono_state.current_index + 1,
                mono_state.candidates.len()
            );
        } else {
            **t = "No targets".to_string();
        }
    }

    let max = mono_state.candidates.len().min(2);
    if let Ok(mut t) = progress_text.single_mut() {
        **t = format!("{} / {} selected", mono_state.selected.len(), max);
    }

    let is_sel = mono_state.is_current_selected();
    if let Ok((mut bg, children)) = toggle_button.single_mut() {
        *bg = if is_sel {
            BackgroundColor(Color::srgb(0.5, 0.2, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.2, 0.4, 0.2))
        };
        for child in children.iter() {
            if let Ok(mut text) = child_texts.get_mut(child) {
                **text = if is_sel { "Spare".to_string() } else { "Eliminate".to_string() };
            }
        }
    }
}

/// Handle button presses in the Monotheism selection panel.
pub fn handle_monotheism_selection_buttons(
    interaction_query: Query<
        (&Interaction, &MonotheismButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut mono_state: ResMut<MonotheismSelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingMonotheismSelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MonotheismButtonAction::Prev => { mono_state.prev(); }
            MonotheismButtonAction::Next => { mono_state.next(); }
            MonotheismButtonAction::Toggle => { mono_state.toggle_current(); }
            MonotheismButtonAction::Confirm => {
                if let Ok(player) = human_waiting.single() {
                    info!(
                        "[MONOTHEISM UI] Human confirmed: {} token(s) eliminated",
                        mono_state.selected.len()
                    );
                    commands.entity(player).remove::<AwaitingMonotheismSelection>();
                }
            }
        }
    }
}

/// Despawn the Monotheism panel when no human is waiting.
pub fn cleanup_monotheism_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<MonotheismSelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingMonotheismSelection>)>,
    mono_state: Res<MonotheismSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && mono_state.player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}
