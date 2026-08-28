use crate::civilization::Z_ACTION_UI;
use crate::civilization::components::GameArea;
use crate::civilization::concepts::check_city_support::check_city_support_components::HasTooManyCities;
use crate::civilization::concepts::check_city_support::check_city_support_events::EliminateCity;
use crate::civilization::concepts::check_city_support::check_city_support_ui_components::*;
use crate::civilization::concepts::map::camera_focus::{
    CameraFocusQueue, focus_camera_on_selection,
};
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;

/// System to detect when the human player has cities they must reduce and
/// populate the selection state, mirroring
/// `setup_human_city_construction_options`.
pub fn setup_human_city_support_options(
    human_players: Query<
        (Entity, &AvailableMoves),
        (With<IsHuman>, With<HasTooManyCities>, Added<AvailableMoves>),
    >,
    mut selection_state: ResMut<CitySupportSelectionState>,
) {
    for (player_entity, available_moves) in human_players.iter() {
        let mut choices: Vec<CitySupportChoice> = Vec::new();
        for game_move in available_moves.moves.values() {
            if let GameMove::EliminateCity(m) = game_move
                && !choices.iter().any(|c| c.area == m.area)
            {
                choices.push(CitySupportChoice {
                    area: m.area,
                    city: m.city,
                    tokens_gained: m.tokens_gained,
                    tokens_needed: m.tokens_needed,
                });
            }
        }

        if !choices.is_empty() {
            info!(
                "[CITY_SUPPORT] Human player has {} cities to choose from for reduction",
                choices.len()
            );
            selection_state.player = Some(player_entity);
            selection_state.choices = choices;
            selection_state.current_index = 0;
        }
    }
}

/// System to draw a highlight on the currently focused city using Gizmos
pub fn draw_city_support_highlight(
    mut gizmos: Gizmos,
    human_players: Query<Entity, (With<IsHuman>, With<HasTooManyCities>, With<AvailableMoves>)>,
    area_transforms: Query<&Transform, With<GameArea>>,
    selection_state: Res<CitySupportSelectionState>,
) {
    if human_players.is_empty() {
        return;
    }

    if let Some(choice) = selection_state.current_choice()
        && let Ok(transform) = area_transforms.get(choice.area)
    {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 35.0, Color::srgb(1.0, 0.2, 0.2));
        gizmos.circle_2d(pos, 38.0, Color::srgb(1.0, 0.2, 0.2));
    }
}

/// System to handle city support button interactions
pub fn handle_city_support_button_clicks(
    interaction_query: Query<
        (&Interaction, &CitySupportButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut selection_state: ResMut<CitySupportSelectionState>,
    mut eliminate_city_writer: MessageWriter<EliminateCity>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            CitySupportButtonAction::EliminateCity => {
                if let (Some(player), Some(choice)) =
                    (selection_state.player, selection_state.current_choice())
                {
                    info!("Human player reducing city to restore city support");
                    eliminate_city_writer.write(EliminateCity::new(
                        player,
                        choice.city,
                        choice.area,
                        false,
                    ));
                    selection_state.clear();
                }
            }
            CitySupportButtonAction::PrevSite => {
                selection_state.prev_site();
            }
            CitySupportButtonAction::NextSite => {
                selection_state.next_site();
            }
        }
    }
}

/// System to spawn the city support controls UI when the human player has
/// cities they must choose from to reduce.
pub fn spawn_city_support_controls_ui(
    mut commands: Commands,
    human_players: Query<Entity, (With<IsHuman>, With<HasTooManyCities>, Added<AvailableMoves>)>,
    existing_ui: Query<Entity, With<CitySupportUiRoot>>,
    asset_server: Res<AssetServer>,
) {
    if !existing_ui.is_empty() {
        return;
    }

    if let Some(_player) = human_players.iter().next() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        commands
            .spawn((
                CitySupportUiRoot,
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
                ZIndex(Z_ACTION_UI),
            ))
            .with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new("CITY SUPPORT: CHOOSE A CITY TO REDUCE"),
                    TextFont {
                        font: FontSource::Handle(font.clone()),
                        font_size: FontSize::Px(22.0),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // Candidate city navigation row
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },))
                    .with_children(|row| {
                        // Prev site button
                        row.spawn((
                            Button,
                            CitySupportButtonAction::PrevSite,
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
                            Text::new("<"),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(24.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Candidate city display
                        row.spawn((
                            CitySupportSiteDisplay,
                            Text::new("City: ?"),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(20.0),
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 0.0)),
                            Node {
                                min_width: Val::Px(260.0),
                                ..default()
                            },
                        ));

                        // Next site button
                        row.spawn((
                            Button,
                            CitySupportButtonAction::NextSite,
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
                            Text::new(">"),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(24.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                // Action button row
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },))
                    .with_children(|row| {
                        row.spawn((
                            Button,
                            CitySupportButtonAction::EliminateCity,
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                        ))
                        .with_child((
                            Text::new("Reduce City"),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(20.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
    }
}

/// System to update the candidate city display text
pub fn update_city_support_site_display(
    selection_state: Res<CitySupportSelectionState>,
    mut text_query: Query<&mut Text, With<CitySupportSiteDisplay>>,
    area_names: Query<&Name, With<GameArea>>,
) {
    if !selection_state.is_changed() {
        return;
    }

    for mut text in &mut text_query {
        if let Some(choice) = selection_state.current_choice() {
            let area_name = area_names
                .get(choice.area)
                .map_or("?", bevy::prelude::Name::as_str);
            **text = format!(
                "{} ({}/{}) — gain {}, need {}",
                area_name,
                selection_state.current_index + 1,
                selection_state.choices.len(),
                choice.tokens_gained,
                choice.tokens_needed,
            );
        } else {
            **text = "No cities to reduce".to_string();
        }
    }
}

/// System to cleanup city support UI when the human player is no longer
/// under-supported.
pub fn cleanup_city_support_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<CitySupportUiRoot>>,
    human_players: Query<Entity, (With<IsHuman>, With<HasTooManyCities>)>,
    mut selection_state: ResMut<CitySupportSelectionState>,
) {
    if human_players.is_empty() {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
        selection_state.clear();
    }
}

/// System to cleanup when exiting a city support activity
pub fn cleanup_city_support_ui_on_exit(
    mut commands: Commands,
    ui_query: Query<Entity, With<CitySupportUiRoot>>,
    mut selection_state: ResMut<CitySupportSelectionState>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
    selection_state.clear();
}

/// Walks the camera to the city the player is currently looking at, so
/// paging through candidate cities pans the board with them.
pub fn focus_camera_on_support_site(
    selection_state: Res<CitySupportSelectionState>,
    area_query: Query<(&Transform, &Name), With<GameArea>>,
    mut focus_queue: ResMut<CameraFocusQueue>,
    mut last_focused: Local<Option<Entity>>,
) {
    let current = selection_state.current_choice().map(|c| c.area);
    if current == *last_focused {
        return;
    }
    *last_focused = current;

    let Some(area) = current else { return };
    if let Ok((transform, name)) = area_query.get(area) {
        focus_camera_on_selection(
            &mut focus_queue,
            transform.translation,
            format!("Reduce city — {name}"),
        );
    }
}
