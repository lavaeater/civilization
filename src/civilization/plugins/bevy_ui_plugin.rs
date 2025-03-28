//! This example illustrates the various features of Bevy UI.

use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy::{
    color::palettes::{basic::LIME, css::DARK_GRAY},
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
    prelude::*,
    ui::widget::NodeImageMode,
    winit::WinitSettings,
};
use std::f32::consts::PI;
use crate::civilization::ui::ui_builder::UIBuilder;

pub struct BevyUiPlugin;

impl Plugin for BevyUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinitSettings::desktop_app())
            .add_plugins(DebugUiPlugin)
            .add_systems(Update, toggle_overlay)
            .add_systems(Startup, setup)
            .add_systems(Update, update_scroll_position);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let bg_color = Color::srgba(0.5, 0.5, 0.5, 0.25);
    let root_ui = UIBuilder::new(commands)
        .block(25.0, 100.0, bg_color)
        .width(Val::Percent(100.0))
        .height(Val::Percent(100.0))
        .text("Hello, UI!", font.clone(), 24.0, Some(Color::WHITE))
        .parent()
        .build();
}

fn toggle_overlay(
    mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>,
) {
    info_once!("Will enable overlays automatically perhaps");
    if !options.enabled {
        options.toggle();
    }
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 20., mouse_wheel_event.y * 20.),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}
