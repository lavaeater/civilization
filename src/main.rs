// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use adv_civ::GamePlugin;
use adv_civ::civilization::{ChronicleFile, create_session_log};
use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::NonSendMarker;
use bevy::log::BoxedLayer;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=warn,bevy_ecs=info,bevy_diagnostic=warn".to_string(),
                    custom_layer: full_log_layer,
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .add_systems(Startup, open_chronicle_file)
        .add_systems(Startup, set_window_icon)
        .run();
}

/// Everything the console shows also goes to `logs/<stamp>-full.log`, so a
/// game that went strange can be looked into afterwards.
fn full_log_layer(_app: &mut App) -> Option<BoxedLayer> {
    let file = create_session_log("full")?;
    Some(Box::new(
        bevy::log::tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(file)),
    ))
}

/// The story of the game, one line per event, in `logs/<stamp>-chronicle.log`.
fn open_chronicle_file(mut commands: Commands) {
    if let Some(file) = create_session_log("chronicle") {
        commands.insert_resource(ChronicleFile(file));
    }
}

// Sets the icon on windows and X11
fn set_window_icon(
    primary_window: Single<Entity, With<PrimaryWindow>>,
    _non_send_marker: NonSendMarker,
) -> Result {
    WINIT_WINDOWS.with_borrow(|windows| {
        let Some(primary) = windows.get_window(*primary_window) else {
            return Err(BevyError::from("No primary window!"));
        };
        let icon_buf = Cursor::new(include_bytes!(
            "../build/macos/AppIcon.iconset/icon_256x256.png"
        ));
        if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            let icon = Icon::from_rgba(rgba, width, height).unwrap();
            primary.set_window_icon(Some(icon));
        }

        Ok(())
    })
}
