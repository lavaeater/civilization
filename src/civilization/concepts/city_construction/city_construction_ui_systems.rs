use crate::civilization::components::GameArea;
use crate::civilization::concepts::city_construction::city_construction_components::IsBuilding;
use crate::civilization::concepts::city_construction::city_construction_events::{
    BuildCityCommand, EndPlayerCityConstruction,
};
use crate::civilization::concepts::city_construction::city_construction_ui_components::*;
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;

/// System to detect when human player has city construction options and populate the selection state
pub fn setup_human_city_construction_options(
    human_players: Query<
        (Entity, &AvailableMoves),
        (
            With<IsHuman>,
            With<IsBuilding>,
            Added<AvailableMoves>,
        ),
    >,
    mut selection_state: ResMut<CityConstructionSelectionState>,
) {
    for (player_entity, available_moves) in human_players.iter() {
        let mut build_sites: Vec<Entity> = Vec::new();
        for game_move in available_moves.moves.values() {
            if let GameMove::CityConstruction(build_city_move) = game_move {
                if !build_sites.contains(&build_city_move.target) {
                    build_sites.push(build_city_move.target);
                }
            }
        }

        if !build_sites.is_empty() {
            info!("Human player has {} city build sites", build_sites.len());
            selection_state.player = Some(player_entity);
            selection_state.build_sites = build_sites;
            selection_state.current_site_index = 0;
        }
    }
}

/// System to draw a highlight on the currently focused build site using Gizmos
pub fn draw_city_construction_highlight(
    mut gizmos: Gizmos,
    human_players: Query<Entity, (With<IsHuman>, With<IsBuilding>, With<AvailableMoves>)>,
    area_transforms: Query<&Transform, With<GameArea>>,
    selection_state: Res<CityConstructionSelectionState>,
) {
    if human_players.is_empty() {
        return;
    }

    if let Some(site) = selection_state.current_site() {
        if let Ok(transform) = area_transforms.get(site) {
            let pos = transform.translation.truncate();
            gizmos.circle_2d(pos, 35.0, Color::srgb(1.0, 0.8, 0.0));
            gizmos.circle_2d(pos, 38.0, Color::srgb(1.0, 0.8, 0.0));
        }
    }
}

/// System to handle city construction button interactions
pub fn handle_city_construction_button_clicks(
    interaction_query: Query<
        (&Interaction, &CityConstructionButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut selection_state: ResMut<CityConstructionSelectionState>,
    mut build_city_writer: MessageWriter<BuildCityCommand>,
    mut end_construction_writer: MessageWriter<EndPlayerCityConstruction>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            CityConstructionButtonAction::BuildCity => {
                if let (Some(player), Some(site)) =
                    (selection_state.player, selection_state.current_site())
                {
                    info!("Human player building city at site");
                    build_city_writer.write(BuildCityCommand::new(player, site));
                    selection_state.clear();
                }
            }
            CityConstructionButtonAction::SkipCity => {
                if let Some(player) = selection_state.player {
                    info!("Human player skipping city construction");
                    end_construction_writer.write(EndPlayerCityConstruction::new(player));
                    selection_state.clear();
                }
            }
            CityConstructionButtonAction::PrevSite => {
                selection_state.prev_site();
            }
            CityConstructionButtonAction::NextSite => {
                selection_state.next_site();
            }
        }
    }
}

/// System to spawn the city construction controls UI when human player has build options
pub fn spawn_city_construction_controls_ui(
    mut commands: Commands,
    human_players: Query<
        Entity,
        (
            With<IsHuman>,
            With<IsBuilding>,
            Added<AvailableMoves>,
        ),
    >,
    existing_ui: Query<Entity, With<CityConstructionUiRoot>>,
    asset_server: Res<AssetServer>,
) {
    if !existing_ui.is_empty() {
        return;
    }

    if let Some(_player) = human_players.iter().next() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        commands
            .spawn((
                CityConstructionUiRoot,
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
                    Text::new("CITY CONSTRUCTION"),
                    TextFont {
                        font: font.clone(),
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.8, 0.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // Build site navigation row
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
                            CityConstructionButtonAction::PrevSite,
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
                                font: font.clone(),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Build site name display
                        row.spawn((
                            BuildSiteDisplay,
                            Text::new("Site: ?"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 0.0)),
                            Node {
                                min_width: Val::Px(200.0),
                                ..default()
                            },
                        ));

                        // Next site button
                        row.spawn((
                            Button,
                            CityConstructionButtonAction::NextSite,
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
                                font: font.clone(),
                                font_size: 24.0,
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
                        // Build button
                        row.spawn((
                            Button,
                            CityConstructionButtonAction::BuildCity,
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                        ))
                        .with_child((
                            Text::new("Build"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Skip / End Construction button
                        row.spawn((
                            Button,
                            CityConstructionButtonAction::SkipCity,
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.2, 0.4)),
                        ))
                        .with_child((
                            Text::new("Skip"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
    }
}

/// System to update the build site display text
pub fn update_build_site_display(
    selection_state: Res<CityConstructionSelectionState>,
    mut text_query: Query<&mut Text, With<BuildSiteDisplay>>,
    area_names: Query<&Name, With<GameArea>>,
) {
    if !selection_state.is_changed() {
        return;
    }

    for mut text in text_query.iter_mut() {
        if let Some(site) = selection_state.current_site() {
            let area_name = area_names.get(site).map(|n| n.as_str()).unwrap_or("?");
            **text = format!(
                "{} ({}/{})",
                area_name,
                selection_state.current_site_index + 1,
                selection_state.build_sites.len()
            );
        } else {
            **text = "No build sites".to_string();
        }
    }
}

/// System to cleanup city construction UI when human player finishes
pub fn cleanup_city_construction_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<CityConstructionUiRoot>>,
    human_players: Query<Entity, (With<IsHuman>, With<IsBuilding>)>,
    mut selection_state: ResMut<CityConstructionSelectionState>,
) {
    if human_players.is_empty() {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
        selection_state.clear();
    }
}

/// System to cleanup when exiting city construction activity
pub fn cleanup_city_construction_ui_on_exit(
    mut commands: Commands,
    ui_query: Query<Entity, With<CityConstructionUiRoot>>,
    mut selection_state: ResMut<CityConstructionSelectionState>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
    selection_state.clear();
}
