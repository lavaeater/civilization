use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

use crate::civilization::components::GameArea;
use crate::civilization::concepts::map::CameraFocusQueue;
use crate::civilization::concepts::ships::ship_ui_components::*;
use crate::stupid_ai::IsHuman;

/// Spawn the ship construction panel when a human player has `AwaitingShipPlacement`.
pub fn spawn_ship_construction_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingShipPlacement>)>,
    existing_ui: Query<Entity, With<ShipConstructionUiRoot>>,
    ship_state: Res<ShipConstructionState>,
    commands: Commands,
    theme: Res<LavaTheme>,
) {
    if !existing_ui.is_empty() || human_waiting.iter().next().is_none() {
        return;
    }

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<ShipConstructionUiRoot>()
        .absolute_position()
        .bottom(Val::Px(20.0))
        .left(Val::Percent(50.0))
        .display_flex()
        .flex_column()
        .align_items_center()
        .padding_all_px(8.0)
        .gap_px(4.0)
        .bg_color(Color::srgba(0.05, 0.1, 0.2, 0.93))
        .z_index(10);

    // Title
    ui.add_text_child(
        "Ship Construction",
        Some(TextStyle::size_color(13.0, Color::srgb(0.4, 0.8, 1.0))),
    );

    // Count row: [−] N ships (max N) [+]
    let initial_count = format!(
        "0 ships  (max {})",
        ship_state.max_buildable
    );
    ui.add_row(|row| {
        row.align_items_center().gap_px(6.0);

        row.add_button_observe(
            "−",
            |btn| { btn.size_px(22.0, 22.0).font_size(12.0); },
            |_: On<Activate>, mut s: ResMut<ShipConstructionState>| { s.decrement(); },
        );

        row.with_child(|c| {
            c.component::<ShipCountText>()
                .with_text(initial_count, Some(TextStyle::size(12.0)))
                .width_px(120.0);
        });

        row.add_button_observe(
            "+",
            |btn| { btn.size_px(22.0, 22.0).font_size(12.0); },
            |_: On<Activate>, mut s: ResMut<ShipConstructionState>| { s.increment(); },
        );
    });

    // Slot row: [◄] Ship N/N [►]
    ui.add_row(|row| {
        row.align_items_center().gap_px(4.0);

        row.add_button_observe(
            "◄",
            |btn| { btn.size_px(20.0, 20.0).font_size(11.0); },
            |_: On<Activate>, mut s: ResMut<ShipConstructionState>| { s.prev_slot(); },
        );

        row.with_child(|c| {
            c.component::<ShipSlotText>()
                .with_text("—", Some(TextStyle::size(12.0)))
                .width_px(65.0);
        });

        row.add_button_observe(
            "►",
            |btn| { btn.size_px(20.0, 20.0).font_size(11.0); },
            |_: On<Activate>, mut s: ResMut<ShipConstructionState>| { s.next_slot(); },
        );
    });

    // Area row: [<] area name [>]
    ui.add_row(|row| {
        row.align_items_center().gap_px(4.0);

        row.add_button_observe(
            "<",
            |btn| { btn.size_px(22.0, 22.0).font_size(12.0); },
            |_: On<Activate>, mut s: ResMut<ShipConstructionState>| { s.prev_area(); },
        );

        row.with_child(|c| {
            c.component::<ShipAreaText>()
                .with_text("no area", Some(TextStyle::size(12.0)))
                .width_px(140.0);
        });

        row.add_button_observe(
            ">",
            |btn| { btn.size_px(22.0, 22.0).font_size(12.0); },
            |_: On<Activate>, mut s: ResMut<ShipConstructionState>| { s.next_area(); },
        );
    });

    // Confirm button — removes the waiting marker to unblock the advance system
    ui.add_button_observe(
        "Confirm",
        |btn| { btn.size_px(110.0, 28.0).font_size(12.0); },
        |_: On<Activate>,
         mut commands: Commands,
         ship_state: Res<ShipConstructionState>,
         human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingShipPlacement>)>| {
            if let Ok(player) = human_waiting.single() {
                info!(
                    "[SHIPS UI] Human confirmed: build {} ships",
                    ship_state.ships_to_build
                );
                commands.entity(player).remove::<AwaitingShipPlacement>();
            }
        },
    );

    ui.build();
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

/// Draw a gizmo highlight on the area currently selected for ship placement.
pub fn draw_ship_construction_highlight(
    mut gizmos: Gizmos,
    ship_state: Res<ShipConstructionState>,
    area_transforms: Query<&Transform, With<GameArea>>,
) {
    if ship_state.ships_to_build == 0 {
        return;
    }
    if let Some(area) = ship_state.current_area()
        && let Ok(transform) = area_transforms.get(area)
    {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 35.0, Color::srgb(0.3, 0.7, 1.0));
        gizmos.circle_2d(pos, 38.0, Color::srgb(0.3, 0.7, 1.0));
    }
}

/// Move the camera to focus on the currently selected ship placement area when
/// the selection changes.
pub fn focus_camera_on_ship_area(
    ship_state: Res<ShipConstructionState>,
    area_transforms: Query<&Transform, With<GameArea>>,
    mut focus_queue: ResMut<CameraFocusQueue>,
) {
    if !ship_state.is_changed() {
        return;
    }
    if ship_state.ships_to_build == 0 {
        return;
    }
    if let Some(area) = ship_state.current_area()
        && let Ok(transform) = area_transforms.get(area)
    {
        focus_queue.add_focus(
            transform.translation,
            0.5,
            "ship construction area",
        );
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
