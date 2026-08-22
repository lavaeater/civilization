use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

use crate::GameState;
use crate::civilization::Z_DIALOG;
use crate::civilization::concepts::map_editor::map_editor_components::*;

#[derive(Component, Default)]
pub struct MapEditorUiRoot;

#[derive(Component, Default)]
pub struct AreaHeaderText;

#[derive(Component, Default)]
pub struct PopulationText;

#[derive(Component)]
pub struct FlagButton(AreaFlag);

#[derive(Component, Default)]
pub struct StartAreaText;

#[derive(Component, Default)]
pub struct ConnectionsText;

#[derive(Component, Default)]
pub struct StatusText;

#[derive(Component, Default)]
pub struct SaveButtonMarker;

pub fn spawn_map_editor_ui(commands: Commands, theme: Res<LavaTheme>) {
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<MapEditorUiRoot>()
        .absolute_position()
        .top(Val::Px(10.0))
        .left(Val::Px(10.0))
        .display_flex()
        .flex_column()
        .padding_all_px(10.0)
        .gap_px(6.0)
        .width_px(320.0)
        .z_index(Z_DIALOG)
        .bg_color(Color::srgba(0.05, 0.1, 0.15, 0.93));

    ui.add_text_child(
        "Map Editor",
        Some(TextStyle::size_color(22.0, Color::srgb(0.4, 0.8, 1.0))),
    );

    ui.with_child(|c| {
        c.component::<AreaHeaderText>().with_text(
            "Area: -",
            Some(TextStyle::size_color(16.0, Color::srgb(1.0, 1.0, 0.7))),
        );
    });

    ui.add_row(|row| {
        row.align_items_center().gap_px(8.0);
        row.add_button_observe(
            "< Prev",
            |btn| {
                btn.size_px(90.0, 32.0);
            },
            |_: On<Activate>,
             mut state: ResMut<MapEditorState>,
             areas: Query<(Entity, &EditableArea)>| {
                step_area(&mut state, &areas, -1);
            },
        );
        row.add_button_observe(
            "Next >",
            |btn| {
                btn.size_px(90.0, 32.0);
            },
            |_: On<Activate>,
             mut state: ResMut<MapEditorState>,
             areas: Query<(Entity, &EditableArea)>| {
                step_area(&mut state, &areas, 1);
            },
        );
    });

    ui.add_text_child(
        "Max population",
        Some(TextStyle::size_color(13.0, Color::srgb(0.7, 0.7, 0.7))),
    );
    ui.add_row(|row| {
        row.align_items_center().gap_px(6.0);
        row.add_button_observe(
            "-1",
            |btn| {
                btn.size_px(36.0, 28.0);
            },
            |_: On<Activate>, state: Res<MapEditorState>, mut areas: Query<&mut EditableArea>| {
                adjust_population(&state, &mut areas, -1);
            },
        );
        row.with_child(|c| {
            c.component::<PopulationText>()
                .with_text(
                    "-",
                    Some(TextStyle::size_color(16.0, Color::srgb(1.0, 1.0, 0.7))),
                )
                .width_px(50.0);
        });
        row.add_button_observe(
            "+1",
            |btn| {
                btn.size_px(36.0, 28.0);
            },
            |_: On<Activate>, state: Res<MapEditorState>, mut areas: Query<&mut EditableArea>| {
                adjust_population(&state, &mut areas, 1);
            },
        );
    });

    ui.add_text_child(
        "Terrain",
        Some(TextStyle::size_color(13.0, Color::srgb(0.7, 0.7, 0.7))),
    );
    ui.add_row(|row| {
        row.display_flex().flex_row().gap_px(6.0);
        for (flag, label) in [
            (AreaFlag::Volcano, "Volcano"),
            (AreaFlag::FloodPlain, "Flood"),
        ] {
            row.add_button_observe(
                label,
                move |btn| {
                    btn.size_px(78.0, 28.0).insert(FlagButton(flag));
                },
                move |_: On<Activate>,
                      state: Res<MapEditorState>,
                      mut areas: Query<&mut EditableArea>| {
                    toggle_flag(&state, &mut areas, flag);
                },
            );
        }
    });
    ui.add_row(|row| {
        row.display_flex().flex_row().gap_px(6.0);
        for (flag, label) in [
            (AreaFlag::CityFlood, "City Flood"),
            (AreaFlag::CitySite, "City Site"),
        ] {
            row.add_button_observe(
                label,
                move |btn| {
                    btn.size_px(78.0, 28.0).insert(FlagButton(flag));
                },
                move |_: On<Activate>,
                      state: Res<MapEditorState>,
                      mut areas: Query<&mut EditableArea>| {
                    toggle_flag(&state, &mut areas, flag);
                },
            );
        }
    });

    ui.with_child(|c| {
        c.component::<StartAreaText>().with_text(
            "Start area: -",
            Some(TextStyle::size_color(14.0, Color::srgb(0.9, 0.9, 0.9))),
        );
    });
    ui.add_button_observe(
        "Cycle Start Faction",
        |btn| {
            btn.size_px(200.0, 28.0);
        },
        |_: On<Activate>, state: Res<MapEditorState>, mut areas: Query<&mut EditableArea>| {
            let Some(entity) = state.current_area else {
                return;
            };
            if let Ok(mut area) = areas.get_mut(entity) {
                area.start_area = next_faction_cycle(area.start_area);
            }
        },
    );

    ui.with_child(|c| {
        c.component::<ConnectionsText>().with_text(
            "Land: -  Sea: -",
            Some(TextStyle::size_color(13.0, Color::srgb(0.8, 0.8, 0.8))),
        );
    });
    ui.add_text_child(
        "Click another area's dot to connect. Click the X on an arrow to remove it.",
        Some(TextStyle::size_color(11.0, Color::srgb(0.6, 0.6, 0.6))),
    );
    ui.add_row(|row| {
        row.align_items_center().gap_px(6.0);
        row.add_button_observe(
            "+ Land",
            |btn| {
                btn.size_px(90.0, 28.0);
            },
            |_: On<Activate>, mut state: ResMut<MapEditorState>| {
                state.pending_connection = Some(ConnectionKind::Land);
            },
        );
        row.add_button_observe(
            "+ Sea",
            |btn| {
                btn.size_px(90.0, 28.0);
            },
            |_: On<Activate>, mut state: ResMut<MapEditorState>| {
                state.pending_connection = Some(ConnectionKind::Sea);
            },
        );
        row.add_button_observe(
            "Cancel",
            |btn| {
                btn.size_px(80.0, 28.0);
            },
            |_: On<Activate>, mut state: ResMut<MapEditorState>| {
                state.pending_connection = None;
            },
        );
    });

    ui.with_child(|c| {
        c.component::<StatusText>().with_text(
            "",
            Some(TextStyle::size_color(12.0, Color::srgb(1.0, 0.8, 0.4))),
        );
    });

    ui.add_row(|row| {
        row.align_items_center().gap_px(8.0);
        row.add_button_observe(
            "Save",
            |btn| {
                btn.size_px(90.0, 34.0)
                    .insert(SaveButtonMarker)
                    .bg_color(Color::srgb(0.2, 0.5, 0.2));
            },
            |_: On<Activate>, mut state: ResMut<MapEditorState>| {
                state.save_requested = true;
            },
        );
        row.add_button_observe(
            "Back to Menu",
            |btn| {
                btn.size_px(150.0, 34.0);
            },
            |_: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                next_state.set(GameState::Menu);
            },
        );
    });

    ui.build();
}

fn step_area(state: &mut MapEditorState, areas: &Query<(Entity, &EditableArea)>, delta: i32) {
    let mut sorted: Vec<(Entity, i32)> = areas.iter().map(|(e, a)| (e, a.id)).collect();
    sorted.sort_by_key(|&(_, id)| id);
    if sorted.is_empty() {
        return;
    }
    let current_idx = state
        .current_area
        .and_then(|cur| sorted.iter().position(|&(e, _)| e == cur));
    let next_idx = match current_idx {
        Some(idx) => {
            let len = sorted.len() as i32;
            (((idx as i32 + delta) % len) + len) % len
        }
        None => 0,
    };
    state.current_area = Some(sorted[next_idx as usize].0);
}

fn adjust_population(state: &Res<MapEditorState>, areas: &mut Query<&mut EditableArea>, delta: i32) {
    let Some(entity) = state.current_area else {
        return;
    };
    if let Ok(mut area) = areas.get_mut(entity) {
        area.max_population = (area.max_population as i32 + delta).max(1) as usize;
    }
}

fn toggle_flag(state: &Res<MapEditorState>, areas: &mut Query<&mut EditableArea>, flag: AreaFlag) {
    let Some(entity) = state.current_area else {
        return;
    };
    let Ok(mut area) = areas.get_mut(entity) else {
        return;
    };
    match flag {
        AreaFlag::Volcano => area.volcano = !area.volcano,
        AreaFlag::FloodPlain => area.flood_plain = !area.flood_plain,
        AreaFlag::CityFlood => area.city_flood = !area.city_flood,
        AreaFlag::CitySite => area.city_site = !area.city_site,
    }
}

#[allow(clippy::type_complexity)]
pub fn update_map_editor_ui(
    state: Res<MapEditorState>,
    areas: Query<&EditableArea>,
    mut header: Query<
        &mut Text,
        (
            With<AreaHeaderText>,
            Without<PopulationText>,
            Without<StartAreaText>,
            Without<ConnectionsText>,
            Without<StatusText>,
        ),
    >,
    mut population: Query<
        &mut Text,
        (
            With<PopulationText>,
            Without<AreaHeaderText>,
            Without<StartAreaText>,
            Without<ConnectionsText>,
            Without<StatusText>,
        ),
    >,
    mut start_area: Query<
        &mut Text,
        (
            With<StartAreaText>,
            Without<AreaHeaderText>,
            Without<PopulationText>,
            Without<ConnectionsText>,
            Without<StatusText>,
        ),
    >,
    mut connections: Query<
        &mut Text,
        (
            With<ConnectionsText>,
            Without<AreaHeaderText>,
            Without<PopulationText>,
            Without<StartAreaText>,
            Without<StatusText>,
        ),
    >,
    mut status: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<AreaHeaderText>,
            Without<PopulationText>,
            Without<StartAreaText>,
            Without<ConnectionsText>,
        ),
    >,
    mut flag_buttons: Query<(&FlagButton, &mut BackgroundColor)>,
    mut save_button: Query<&mut BackgroundColor, (With<SaveButtonMarker>, Without<FlagButton>)>,
) {
    let current = state.current_area.and_then(|e| areas.get(e).ok());

    if let Ok(mut t) = header.single_mut() {
        **t = match current {
            Some(a) => {
                let pending = state
                    .pending_connection
                    .map(|k| format!(" [picking {k:?} target...]"))
                    .unwrap_or_default();
                format!("Area: {}{}", a.id, pending)
            }
            None => "Area: -".to_string(),
        };
    }
    if let Ok(mut t) = population.single_mut() {
        **t = current.map_or_else(|| "-".to_string(), |a| a.max_population.to_string());
    }
    if let Ok(mut t) = start_area.single_mut() {
        **t = format!(
            "Start area: {}",
            current
                .and_then(|a| a.start_area)
                .map_or_else(|| "none".to_string(), |f| format!("{f:?}"))
        );
    }
    if let Ok(mut t) = connections.single_mut() {
        **t = current.map_or_else(
            || "Land: -  Sea: -".to_string(),
            |a| format!("Land: {:?}  Sea: {:?}", a.land_connections, a.sea_connections),
        );
    }
    if let Ok(mut t) = status.single_mut() {
        **t = state.status.clone().unwrap_or_default();
    }

    for (flag_button, mut bg) in &mut flag_buttons {
        let active = current.is_some_and(|a| match flag_button.0 {
            AreaFlag::Volcano => a.volcano,
            AreaFlag::FloodPlain => a.flood_plain,
            AreaFlag::CityFlood => a.city_flood,
            AreaFlag::CitySite => a.city_site,
        });
        *bg = BackgroundColor(if active {
            Color::srgb(0.2, 0.55, 0.2)
        } else {
            Color::srgb(0.3, 0.3, 0.3)
        });
    }

    if let Ok(mut bg) = save_button.single_mut() {
        *bg = BackgroundColor(if state.dirty {
            Color::srgb(0.7, 0.5, 0.1)
        } else {
            Color::srgb(0.2, 0.5, 0.2)
        });
    }
}

pub fn cleanup_map_editor_ui(mut commands: Commands, ui_root: Query<Entity, With<MapEditorUiRoot>>) {
    for entity in &ui_root {
        commands.entity(entity).despawn();
    }
}
