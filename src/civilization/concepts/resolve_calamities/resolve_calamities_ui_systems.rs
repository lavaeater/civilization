use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{InteractionPalette, LavaTheme, TextStyle, UIBuilder};

use crate::civilization::components::GameArea;
use crate::civilization::concepts::resolve_calamities::calamities::civil_war::FactionChoice;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::*;
use crate::civilization::concepts::map::camera_focus::{focus_camera_on_selection, CameraFocusQueue};
use crate::civilization::Z_DIALOG;
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
            ZIndex(Z_DIALOG),
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
    mut child_texts: Query<&mut Text, (Without<CalamitySelectionTitleText>, Without<CalamitySelectionCityNameText>, Without<CalamitySelectionProgressText>)>,
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
            let name = area_names.get(city).map_or("?", bevy::prelude::Name::as_str);
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

    if cw_selection.role == CivilWarUiRole::ChooseFaction {
        spawn_civil_war_faction_choice_ui(&mut commands, &cw_selection, font);
        return;
    }

    let role_label = match cw_selection.role {
        CivilWarUiRole::Victim => "Civil War — Victim Selection",
        CivilWarUiRole::Beneficiary => "Civil War — Beneficiary Selection",
        CivilWarUiRole::ChooseFaction => unreachable!("handled above"),
    };
    let hint = match cw_selection.role {
        CivilWarUiRole::Victim => format!("Select at least {} pts to yield", cw_selection.target_points),
        CivilWarUiRole::Beneficiary => format!("Take up to {} pts from the pool", cw_selection.target_points),
        CivilWarUiRole::ChooseFaction => unreachable!("handled above"),
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
            ZIndex(Z_DIALOG),
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

/// Rule 30.415: minimal two-button panel letting the primary victim pick
/// which of the two finalized factions to keep.
fn spawn_civil_war_faction_choice_ui(
    commands: &mut Commands,
    cw_selection: &CivilWarSelectionState,
    font: Handle<Font>,
) {
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
            ZIndex(Z_DIALOG),
        ))
        .with_children(|parent| {
            parent.spawn((
                CivilWarTitleText,
                Text::new("Civil War — Choose Your Faction"),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(1.0, 0.5, 0.3)),
            ));
            parent.spawn((
                Text::new("Whichever faction you don't keep is annexed by the beneficiary"),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        CivilWarButtonAction::KeepFirstFaction,
                        CivilWarKeepFirstButton,
                        Node {
                            width: Val::Px(180.0), height: Val::Px(40.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                    ))
                    .with_child((
                        Text::new(format!("Keep First Faction ({} pts)", cw_selection.first_faction_points)),
                        TextFont { font: font.clone(), font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    row.spawn((
                        Button,
                        CivilWarButtonAction::KeepSecondFaction,
                        CivilWarKeepSecondButton,
                        Node {
                            width: Val::Px(180.0), height: Val::Px(40.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.5)),
                    ))
                    .with_child((
                        Text::new(format!("Keep Second Faction ({} pts)", cw_selection.second_faction_points)),
                        TextFont { font: font.clone(), font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
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
    if cw_selection.role == CivilWarUiRole::ChooseFaction { return; }

    if let Ok(mut text) = points_text.single_mut() {
        let pts = cw_selection.current_points();
        let target = cw_selection.target_points;
        let label = match cw_selection.role {
            CivilWarUiRole::Victim => format!("Points: {pts} / {target} (need ≥{target})"),
            CivilWarUiRole::Beneficiary => format!("Points: {pts} / {target} (take up to {target})"),
            CivilWarUiRole::ChooseFaction => String::new(),
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

    if let Ok(mut text) = city_text.single_mut()
        && let Some(city) = cw_selection.current_city()
    {
        let name = area_names.get(city).map_or("?", bevy::prelude::Name::as_str);
        let sel = if cw_selection.is_current_city_selected() { " [✓]" } else { "" };
        **text = format!(
            "{}{} ({}/{})",
            name, sel,
            cw_selection.current_city_index + 1,
            cw_selection.available_cities.len()
        );
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
                if cw_selection.selection_valid()
                    && let Ok(player) = human_waiting.single()
                {
                    info!("[CIVIL WAR UI] Human confirmed: {} tokens, {} cities",
                        cw_selection.selected_token_count, cw_selection.selected_cities.len());
                    commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                }
            }
            CivilWarButtonAction::KeepFirstFaction => {
                if let Ok(player) = human_waiting.single() {
                    info!("[CIVIL WAR UI] Human victim keeps First faction (30.415)");
                    cw_selection.choose_faction(FactionChoice::First);
                    commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                }
            }
            CivilWarButtonAction::KeepSecondFaction => {
                if let Ok(player) = human_waiting.single() {
                    info!("[CIVIL WAR UI] Human victim keeps Second faction (30.415)");
                    cw_selection.choose_faction(FactionChoice::Second);
                    commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
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
    commands: Commands,
    theme: Res<LavaTheme>,
) {
    if !existing_ui.is_empty() || mono_state.player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let max = mono_state.candidates.len().min(2);

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<MonotheismSelectionUiRoot>()
        .absolute_position()
        .bottom(Val::Px(20.0))
        .left(Val::Percent(50.0))
        .display_flex()
        .flex_column()
        .align_items_center()
        .padding_all_px(10.0)
        .gap_px(8.0)
        .z_index(Z_DIALOG)
        .bg_color(Color::srgba(0.05, 0.1, 0.15, 0.93));

    // Title
    ui.add_text_child(
        "Monotheism",
        Some(TextStyle::size_color(22.0, Color::srgb(0.4, 0.8, 1.0))),
    );
    ui.add_text_child(
        format!(
            "Select up to {} enemy token{} to eliminate",
            max,
            if max == 1 { "" } else { "s" }
        ),
        Some(TextStyle::size_color(14.0, Color::srgb(0.7, 0.7, 0.7))),
    );

    // Target navigation row: [<] target text [>]
    ui.add_row(|row| {
        row.align_items_center().gap_px(8.0);

        row.add_button_observe(
            "<",
            |btn| { btn.size_px(32.0, 32.0); },
            |_: On<Activate>, mut s: ResMut<MonotheismSelectionState>| { s.prev(); },
        );

        row.with_child(|c| {
            c.component::<MonotheismTargetText>()
                .with_text("?", Some(TextStyle::size_color(18.0, Color::srgb(1.0, 1.0, 0.7))))
                .width_px(200.0);
        });

        row.add_button_observe(
            ">",
            |btn| { btn.size_px(32.0, 32.0); },
            |_: On<Activate>, mut s: ResMut<MonotheismSelectionState>| { s.next(); },
        );
    });

    // Toggle + progress row
    let toggle_color = Color::srgb(0.2, 0.4, 0.2);
    ui.add_row(|row| {
        row.align_items_center().gap_px(12.0);

        row.add_button_observe(
            "Eliminate",
            |btn| {
                btn.size_px(110.0, 36.0)
                    .insert(MonotheismToggleButton)
                    .bg_color(toggle_color);
            },
            |_: On<Activate>, mut s: ResMut<MonotheismSelectionState>| { s.toggle_current(); },
        );

        row.with_child(|c| {
            c.component::<MonotheismProgressText>()
                .with_text(
                    format!("0 / {max}"),
                    Some(TextStyle::size_color(18.0, Color::srgb(0.8, 0.8, 0.8))),
                );
        });
    });

    // Confirm button (0 selections is valid — "do nothing")
    ui.add_button_observe(
        "Confirm",
        |btn| { btn.size_px(160.0, 40.0); },
        |_: On<Activate>,
         mut commands: Commands,
         human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingMonotheismSelection>)>,
         mono_state: Res<MonotheismSelectionState>| {
            if let Ok(player) = human_waiting.single() {
                info!(
                    "[MONOTHEISM UI] Human confirmed: {} token(s) eliminated",
                    mono_state.selected.len()
                );
                commands.entity(player).remove::<AwaitingMonotheismSelection>();
            }
        },
    );

    ui.build();
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
    mut toggle_button: Query<
        (&mut BackgroundColor, &mut InteractionPalette, &Children),
        With<MonotheismToggleButton>,
    >,
    mut child_texts: Query<
        &mut Text,
        (Without<MonotheismTargetText>, Without<MonotheismProgressText>),
    >,
) {
    if !mono_state.is_changed() {
        return;
    }

    if let Ok(mut t) = target_text.single_mut() {
        if let Some((_, area)) = mono_state.current_candidate() {
            let area_name = area_names.get(area).map_or("?", bevy::prelude::Name::as_str);
            let sel = if mono_state.is_current_selected() { " [✓]" } else { "" };
            **t = format!(
                "{}{} ({}/{})",
                area_name,
                sel,
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
    let toggle_color = if is_sel {
        Color::srgb(0.5, 0.2, 0.2)
    } else {
        Color::srgb(0.2, 0.4, 0.2)
    };
    if let Ok((mut bg, mut palette, children)) = toggle_button.single_mut() {
        *bg = BackgroundColor(toggle_color);
        // Also update the palette's none-state so the color sticks when not hovered
        palette.none = toggle_color;
        for child in children.iter() {
            if let Ok(mut text) = child_texts.get_mut(child) {
                **text = if is_sel { "Spare".to_string() } else { "Eliminate".to_string() };
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

// ── Flood secondary-victim allocation UI (rule 30.512) ──────────────────────

/// Spawn the Flood allocation panel when a human primary victim has
/// `AwaitingHumanCalamitySelection` and `FloodSelectionState` has an acting
/// player set.
pub fn spawn_flood_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingHumanCalamitySelection>)>,
    existing_ui: Query<Entity, With<FloodSelectionUiRoot>>,
    flood_selection: Res<FloodSelectionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !existing_ui.is_empty() || flood_selection.acting_player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            FloodSelectionUiRoot,
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
            BackgroundColor(Color::srgba(0.05, 0.1, 0.15, 0.93)),
            ZIndex(Z_DIALOG),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Flood — Divide the secondary loss"),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.4, 0.7, 1.0)),
            ));

            parent.spawn((
                Text::new("Choose how many points each secondary victim loses"),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                FloodPointsText,
                Text::new("Allocated: 0 / ?"),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));

            // Victim navigation + allocation row: [<] Name: N (of M) [>]  [-] [+]
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button, FloodButtonAction::PrevVictim,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        FloodVictimNameText,
                        Text::new("Victim: ?"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(200.0), ..default() },
                    ));

                    row.spawn((
                        Button, FloodButtonAction::NextVictim,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, FloodButtonAction::Decrement,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                    )).with_child((Text::new("−"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, FloodButtonAction::Increment,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.1, 0.3, 0.1)),
                    )).with_child((Text::new("+"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                });

            parent.spawn((
                Button,
                FloodButtonAction::Confirm,
                FloodConfirmButton,
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

/// Update Flood allocation UI text each frame the state changes.
pub fn update_flood_selection_ui(
    flood_selection: Res<FloodSelectionState>,
    player_names: Query<&Name>,
    mut points_text: Query<&mut Text, (With<FloodPointsText>, Without<FloodVictimNameText>)>,
    mut victim_text: Query<&mut Text, (With<FloodVictimNameText>, Without<FloodPointsText>)>,
    mut confirm_button: Query<&mut BackgroundColor, With<FloodConfirmButton>>,
) {
    if !flood_selection.is_changed() {
        return;
    }

    if let Ok(mut text) = points_text.single_mut() {
        **text = format!(
            "Allocated: {} / {}",
            flood_selection.allocated_total(),
            flood_selection.total_budget
        );
    }

    if let Ok(mut text) = victim_text.single_mut()
        && let Some((victim, available, allocated)) = flood_selection.current_victim()
    {
        let name = player_names.get(victim).map_or("?", Name::as_str);
        **text = format!(
            "{}: {} (of {})  [{}/{}]",
            name,
            allocated,
            available,
            flood_selection.current_victim_index + 1,
            flood_selection.victims.len()
        );
    }

    if let Ok(mut bg) = confirm_button.single_mut() {
        *bg = if flood_selection.selection_valid() {
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25))
        };
    }
}

/// Handle Flood allocation UI button presses.
pub fn handle_flood_selection_buttons(
    interaction_query: Query<
        (&Interaction, &FloodButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut flood_selection: ResMut<FloodSelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            FloodButtonAction::PrevVictim => { flood_selection.prev_victim(); }
            FloodButtonAction::NextVictim => { flood_selection.next_victim(); }
            FloodButtonAction::Increment => { flood_selection.increment_current(); }
            FloodButtonAction::Decrement => { flood_selection.decrement_current(); }
            FloodButtonAction::Confirm => {
                if flood_selection.selection_valid()
                    && let Ok(player) = human_waiting.single()
                {
                    info!(
                        "[FLOOD UI] Human primary victim confirmed allocation: {:?}",
                        flood_selection.victims
                    );
                    commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                }
            }
        }
    }
}

/// Despawn Flood allocation UI when no human is waiting and acting_player is cleared.
pub fn cleanup_flood_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<FloodSelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
    flood_selection: Res<FloodSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && flood_selection.acting_player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Famine secondary-victim allocation UI (rule 30.311) ──────────────────────

/// Spawn the Famine allocation panel when a human primary victim has
/// `AwaitingHumanCalamitySelection` and `FamineSelectionState` has an acting
/// player set.
pub fn spawn_famine_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingHumanCalamitySelection>)>,
    existing_ui: Query<Entity, With<FamineSelectionUiRoot>>,
    famine_selection: Res<FamineSelectionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !existing_ui.is_empty() || famine_selection.acting_player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            FamineSelectionUiRoot,
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
            BackgroundColor(Color::srgba(0.15, 0.1, 0.05, 0.93)),
            ZIndex(Z_DIALOG),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Famine — Divide the secondary loss"),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(1.0, 0.6, 0.3)),
            ));

            parent.spawn((
                Text::new("Choose how many points each secondary victim loses"),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                FaminePointsText,
                Text::new("Allocated: 0 / ?"),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));

            // Victim navigation + allocation row: [<] Name: N (of M) [>]  [-] [+]
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button, FamineButtonAction::PrevVictim,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        FamineVictimNameText,
                        Text::new("Victim: ?"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(200.0), ..default() },
                    ));

                    row.spawn((
                        Button, FamineButtonAction::NextVictim,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, FamineButtonAction::Decrement,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                    )).with_child((Text::new("−"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, FamineButtonAction::Increment,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.1, 0.3, 0.1)),
                    )).with_child((Text::new("+"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                });

            parent.spawn((
                Button,
                FamineButtonAction::Confirm,
                FamineConfirmButton,
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

/// Update Famine allocation UI text each frame the state changes.
pub fn update_famine_selection_ui(
    famine_selection: Res<FamineSelectionState>,
    player_names: Query<&Name>,
    mut points_text: Query<&mut Text, (With<FaminePointsText>, Without<FamineVictimNameText>)>,
    mut victim_text: Query<&mut Text, (With<FamineVictimNameText>, Without<FaminePointsText>)>,
    mut confirm_button: Query<&mut BackgroundColor, With<FamineConfirmButton>>,
) {
    if !famine_selection.is_changed() {
        return;
    }

    if let Ok(mut text) = points_text.single_mut() {
        **text = format!(
            "Allocated: {} / {}",
            famine_selection.allocated_total(),
            famine_selection.total_budget
        );
    }

    if let Ok(mut text) = victim_text.single_mut()
        && let Some((victim, available, allocated)) = famine_selection.current_victim()
    {
        let name = player_names.get(victim).map_or("?", Name::as_str);
        **text = format!(
            "{}: {} (of {})  [{}/{}]",
            name,
            allocated,
            available,
            famine_selection.current_victim_index + 1,
            famine_selection.victims.len()
        );
    }

    if let Ok(mut bg) = confirm_button.single_mut() {
        *bg = if famine_selection.selection_valid() {
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25))
        };
    }
}

/// Handle Famine allocation UI button presses.
pub fn handle_famine_selection_buttons(
    interaction_query: Query<
        (&Interaction, &FamineButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut famine_selection: ResMut<FamineSelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            FamineButtonAction::PrevVictim => { famine_selection.prev_victim(); }
            FamineButtonAction::NextVictim => { famine_selection.next_victim(); }
            FamineButtonAction::Increment => { famine_selection.increment_current(); }
            FamineButtonAction::Decrement => { famine_selection.decrement_current(); }
            FamineButtonAction::Confirm => {
                if famine_selection.selection_valid()
                    && let Ok(player) = human_waiting.single()
                {
                    info!(
                        "[FAMINE UI] Human primary victim confirmed allocation: {:?}",
                        famine_selection.victims
                    );
                    commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                }
            }
        }
    }
}

/// Despawn Famine allocation UI when no human is waiting and acting_player is cleared.
pub fn cleanup_famine_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<FamineSelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
    famine_selection: Res<FamineSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && famine_selection.acting_player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Epidemic secondary-victim allocation UI (rule 30.611) ───────────────────

/// Spawn the Epidemic allocation panel when a human primary victim has
/// `AwaitingHumanCalamitySelection` and `EpidemicSelectionState` has an acting
/// player set.
pub fn spawn_epidemic_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingHumanCalamitySelection>)>,
    existing_ui: Query<Entity, With<EpidemicSelectionUiRoot>>,
    epidemic_selection: Res<EpidemicSelectionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !existing_ui.is_empty() || epidemic_selection.acting_player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            EpidemicSelectionUiRoot,
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
            BackgroundColor(Color::srgba(0.05, 0.1, 0.15, 0.93)),
            ZIndex(Z_DIALOG),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Epidemic — Divide the secondary loss"),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.4, 0.7, 1.0)),
            ));

            parent.spawn((
                Text::new("Choose how many points each secondary victim loses"),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                EpidemicPointsText,
                Text::new("Allocated: 0 / ?"),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));

            // Victim navigation + allocation row: [<] Name: N (of M) [>]  [-] [+]
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button, EpidemicButtonAction::PrevVictim,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        EpidemicVictimNameText,
                        Text::new("Victim: ?"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(200.0), ..default() },
                    ));

                    row.spawn((
                        Button, EpidemicButtonAction::NextVictim,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, EpidemicButtonAction::Decrement,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                    )).with_child((Text::new("−"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, EpidemicButtonAction::Increment,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.1, 0.3, 0.1)),
                    )).with_child((Text::new("+"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                });

            parent.spawn((
                Button,
                EpidemicButtonAction::Confirm,
                EpidemicConfirmButton,
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

/// Update Epidemic allocation UI text each frame the state changes.
pub fn update_epidemic_selection_ui(
    epidemic_selection: Res<EpidemicSelectionState>,
    player_names: Query<&Name>,
    mut points_text: Query<&mut Text, (With<EpidemicPointsText>, Without<EpidemicVictimNameText>)>,
    mut victim_text: Query<&mut Text, (With<EpidemicVictimNameText>, Without<EpidemicPointsText>)>,
    mut confirm_button: Query<&mut BackgroundColor, With<EpidemicConfirmButton>>,
) {
    if !epidemic_selection.is_changed() {
        return;
    }

    if let Ok(mut text) = points_text.single_mut() {
        **text = format!(
            "Allocated: {} / {}",
            epidemic_selection.allocated_total(),
            epidemic_selection.total_budget
        );
    }

    if let Ok(mut text) = victim_text.single_mut()
        && let Some((victim, available, allocated)) = epidemic_selection.current_victim()
    {
        let name = player_names.get(victim).map_or("?", Name::as_str);
        **text = format!(
            "{}: {} (of {})  [{}/{}]",
            name,
            allocated,
            available,
            epidemic_selection.current_victim_index + 1,
            epidemic_selection.victims.len()
        );
    }

    if let Ok(mut bg) = confirm_button.single_mut() {
        *bg = if epidemic_selection.selection_valid() {
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25))
        };
    }
}

/// Handle Epidemic allocation UI button presses.
pub fn handle_epidemic_selection_buttons(
    interaction_query: Query<
        (&Interaction, &EpidemicButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut epidemic_selection: ResMut<EpidemicSelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            EpidemicButtonAction::PrevVictim => { epidemic_selection.prev_victim(); }
            EpidemicButtonAction::NextVictim => { epidemic_selection.next_victim(); }
            EpidemicButtonAction::Increment => { epidemic_selection.increment_current(); }
            EpidemicButtonAction::Decrement => { epidemic_selection.decrement_current(); }
            EpidemicButtonAction::Confirm => {
                if epidemic_selection.selection_valid()
                    && let Ok(player) = human_waiting.single()
                {
                    info!(
                        "[EPIDEMIC UI] Human primary victim confirmed allocation: {:?}",
                        epidemic_selection.victims
                    );
                    commands.entity(player).remove::<AwaitingHumanCalamitySelection>();
                }
            }
        }
    }
}

/// Despawn Epidemic allocation UI when no human is waiting and acting_player is cleared.
pub fn cleanup_epidemic_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<EpidemicSelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
    epidemic_selection: Res<EpidemicSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && epidemic_selection.acting_player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Primary unit-point loss selection UI (rules 29.62/29.63) ─────────────────

/// Spawn the "choose which units to lose" panel when a human victim has
/// `AwaitingHumanCalamitySelection` and `UnitLossSelectionState` has an acting
/// player set.
pub fn spawn_unit_loss_selection_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingHumanCalamitySelection>)>,
    existing_ui: Query<Entity, With<UnitLossSelectionUiRoot>>,
    unit_loss: Res<UnitLossSelectionState>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !existing_ui.is_empty() || unit_loss.acting_player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            UnitLossSelectionUiRoot,
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
            BackgroundColor(Color::srgba(0.15, 0.1, 0.05, 0.93)),
            ZIndex(Z_DIALOG),
        ))
        .with_children(|parent| {
            parent.spawn((
                UnitLossTitleText,
                Text::new(format!("{} — Choose your losses", unit_loss.calamity_name)),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::srgb(1.0, 0.6, 0.3)),
            ));

            parent.spawn((
                Text::new("Pick which areas your units are removed from"),
                TextFont { font: font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                UnitLossPointsText,
                Text::new("Assigned: 0 / ?"),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));

            // Area navigation + allocation row: [<] Area: N (of M) [>]  [-] [+]
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button, UnitLossButtonAction::PrevArea,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        UnitLossAreaNameText,
                        Text::new("Area: ?"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.7)),
                        Node { min_width: Val::Px(240.0), ..default() },
                    ));

                    row.spawn((
                        Button, UnitLossButtonAction::NextArea,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, UnitLossButtonAction::Decrement,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                    )).with_child((Text::new("−"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, UnitLossButtonAction::Increment,
                        Node { width: Val::Px(32.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.1, 0.3, 0.1)),
                    )).with_child((Text::new("+"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
                });

            // City row (29.62: a city is worth up to 5 points). Present even
            // when the victim has none, so the panel's shape is stable; the
            // text simply reads that there is nothing to give.
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button, UnitLossButtonAction::PrevCity,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new("<"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        UnitLossCityText,
                        Text::new("City: —"),
                        TextFont { font: font.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.85, 0.85)),
                        Node { min_width: Val::Px(240.0), ..default() },
                    ));

                    row.spawn((
                        Button, UnitLossButtonAction::NextCity,
                        Node { width: Val::Px(28.0), height: Val::Px(28.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_child((Text::new(">"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));

                    row.spawn((
                        Button, UnitLossButtonAction::ToggleCity,
                        Node { width: Val::Px(90.0), height: Val::Px(32.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.35, 0.15, 0.15)),
                    )).with_child((Text::new("Give up"), TextFont { font: font.clone(), font_size: 15.0, ..default() }, TextColor(Color::WHITE)));
                });

            parent.spawn((
                Button,
                UnitLossButtonAction::Confirm,
                UnitLossConfirmButton,
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

/// Update the unit-loss panel's text whenever the selection changes.
pub fn update_unit_loss_selection_ui(
    unit_loss: Res<UnitLossSelectionState>,
    area_names: Query<&Name, With<GameArea>>,
    mut points_text: Query<
        &mut Text,
        (With<UnitLossPointsText>, Without<UnitLossAreaNameText>, Without<UnitLossTitleText>, Without<UnitLossCityText>),
    >,
    mut area_text: Query<
        &mut Text,
        (With<UnitLossAreaNameText>, Without<UnitLossPointsText>, Without<UnitLossTitleText>, Without<UnitLossCityText>),
    >,
    mut title_text: Query<
        &mut Text,
        (With<UnitLossTitleText>, Without<UnitLossPointsText>, Without<UnitLossAreaNameText>, Without<UnitLossCityText>),
    >,
    mut city_text: Query<
        &mut Text,
        (With<UnitLossCityText>, Without<UnitLossPointsText>, Without<UnitLossAreaNameText>, Without<UnitLossTitleText>),
    >,
    mut confirm_button: Query<&mut BackgroundColor, With<UnitLossConfirmButton>>,
) {
    if !unit_loss.is_changed() {
        return;
    }

    if let Ok(mut text) = title_text.single_mut() {
        **text = format!("{} — Choose your losses", unit_loss.calamity_name);
    }

    if let Ok(mut text) = points_text.single_mut() {
        **text = format!(
            "Assigned: {} / {}",
            unit_loss.allocated_total(),
            unit_loss.required_total()
        );
    }

    if let Ok(mut text) = area_text.single_mut()
        && let Some((area, available, allocated)) = unit_loss.current_area()
    {
        let name = area_names.get(area).map_or("?", Name::as_str);
        **text = format!(
            "{}: lose {} (of {})  [{}/{}]",
            name,
            allocated,
            available,
            unit_loss.current_area_index + 1,
            unit_loss.areas.len()
        );
    }

    if let Ok(mut text) = city_text.single_mut() {
        **text = match unit_loss.current_city() {
            Some((city, selected)) => {
                let name = area_names.get(city).map_or("?", Name::as_str);
                format!(
                    "City in {}: {} ({} pts)  [{}/{}]",
                    name,
                    if selected { "GIVEN UP" } else { "kept" },
                    CITY_UNIT_POINTS,
                    unit_loss.current_city_index + 1,
                    unit_loss.cities.len()
                )
            }
            None => "City: — (none to give)".to_string(),
        };
    }

    if let Ok(mut bg) = confirm_button.single_mut() {
        *bg = if unit_loss.selection_valid() {
            BackgroundColor(Color::srgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25))
        };
    }
}

/// Handle unit-loss panel button presses.
pub fn handle_unit_loss_selection_buttons(
    interaction_query: Query<
        (&Interaction, &UnitLossButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut unit_loss: ResMut<UnitLossSelectionState>,
    mut commands: Commands,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            UnitLossButtonAction::PrevArea => { unit_loss.prev_area(); }
            UnitLossButtonAction::NextArea => { unit_loss.next_area(); }
            UnitLossButtonAction::Increment => { unit_loss.increment_current(); }
            UnitLossButtonAction::Decrement => { unit_loss.decrement_current(); }
            UnitLossButtonAction::PrevCity => { unit_loss.prev_city(); }
            UnitLossButtonAction::NextCity => { unit_loss.next_city(); }
            UnitLossButtonAction::ToggleCity => { unit_loss.toggle_current_city(); }
            UnitLossButtonAction::Confirm => {
                // The acting player is the one that must be released -- other
                // humans may hold the marker for a different calamity.
                if unit_loss.selection_valid()
                    && let Some(acting) = unit_loss.acting_player
                    && human_waiting.iter().any(|e| e == acting)
                {
                    info!(
                        "[UNIT LOSS UI] Human victim confirmed {} point(s) across {} area(s)",
                        unit_loss.allocated_total(),
                        unit_loss.areas.len()
                    );
                    commands.entity(acting).remove::<AwaitingHumanCalamitySelection>();
                }
            }
        }
    }
}

/// Despawn the unit-loss panel once the choice has been consumed.
pub fn cleanup_unit_loss_selection_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<UnitLossSelectionUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingHumanCalamitySelection>)>,
    unit_loss: Res<UnitLossSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && unit_loss.acting_player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Camera follow for selection panels ───────────────────────────────────────
//
// A panel that names an area is only half an answer -- the player still has to
// find it on the map. These systems walk the camera to whatever the panel is
// currently showing, so scrolling the options scrolls the board.

/// Follows the city/area cursor of the shared calamity selection panel
/// (Superstition, Civil Disorder, Slave Revolt, Treachery, Iconoclasm, Piracy,
/// Flood's coastal fallback, the Volcano tie-break, the Barbarian tie-break).
pub fn focus_camera_on_calamity_selection(
    calamity_selection: Res<CalamitySelectionState>,
    area_query: Query<(&Transform, &Name), With<GameArea>>,
    mut focus_queue: ResMut<CameraFocusQueue>,
    mut last_focused: Local<Option<Entity>>,
) {
    let current = calamity_selection.current_city();
    if current == *last_focused {
        return;
    }
    *last_focused = current;

    let Some(area) = current else { return };
    if let Ok((transform, name)) = area_query.get(area) {
        focus_camera_on_selection(
            &mut focus_queue,
            transform.translation,
            format!("{} — {}", calamity_selection.calamity_name, name),
        );
    }
}

/// Follows both cursors of the unit-loss panel: the area whose tokens are
/// being assigned, and the city being considered for surrender.
pub fn focus_camera_on_unit_loss_selection(
    unit_loss: Res<UnitLossSelectionState>,
    area_query: Query<(&Transform, &Name), With<GameArea>>,
    mut focus_queue: ResMut<CameraFocusQueue>,
    mut last_area: Local<Option<Entity>>,
    mut last_city: Local<Option<Entity>>,
) {
    let current_area = unit_loss.current_area().map(|(area, _, _)| area);
    let current_city = unit_loss.current_city().map(|(city, _)| city);

    // Whichever cursor the player just moved is the one they are looking at.
    let moved = if current_city != *last_city {
        current_city
    } else if current_area != *last_area {
        current_area
    } else {
        None
    };
    *last_area = current_area;
    *last_city = current_city;

    let Some(area) = moved else { return };
    if let Ok((transform, name)) = area_query.get(area) {
        focus_camera_on_selection(
            &mut focus_queue,
            transform.translation,
            format!("{} — {}", unit_loss.calamity_name, name),
        );
    }
}

/// Follows the Monotheism conversion cursor to the area holding the token
/// currently under consideration (rule 32.94).
pub fn focus_camera_on_monotheism_selection(
    mono_state: Res<MonotheismSelectionState>,
    area_query: Query<(&Transform, &Name), With<GameArea>>,
    mut focus_queue: ResMut<CameraFocusQueue>,
    mut last_focused: Local<Option<Entity>>,
) {
    let current = mono_state
        .candidates
        .get(mono_state.current_index)
        .map(|&(_, area)| area);
    if current == *last_focused {
        return;
    }
    *last_focused = current;

    let Some(area) = current else { return };
    if let Ok((transform, name)) = area_query.get(area) {
        focus_camera_on_selection(
            &mut focus_queue,
            transform.translation,
            format!("Monotheism — {name}"),
        );
    }
}
