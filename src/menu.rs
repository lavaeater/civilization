use crate::civilization::save_game::{LoadGameRequest, SaveGameRequest};
use crate::civilization::GameCamera;
use crate::loading::TextureAssets;
use crate::{GamePaused, GameState};
use bevy::feathers::FeathersPlugins;
use bevy::{
    feathers::{dark_theme::create_dark_theme, theme::UiTheme},
    prelude::*,
    ui_widgets::Activate,
};
use lava_ui_builder::{LavaTheme, UIBuilder};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()))
            .init_resource::<LavaTheme>()
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

#[derive(Component, Default)]
struct Menu;

#[derive(Component, Default)]
struct PauseMenu;

// ============================================================================
// Main Menu
// ============================================================================

fn setup_menu(mut commands: Commands, _textures: Res<TextureAssets>, theme: Res<LavaTheme>) {
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection::default_2d()),
        GameCamera,
        Msaa::Off,
    ));

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<Menu>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .gap_px(16.0);

    ui.add_text_child("Advanced Civilization", None, Some(48.0), None);

    ui.add_button_observe(
        "Play",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
            info!("Play button clicked!");
            next_state.set(GameState::Playing);
        },
    );

    ui.add_button_observe(
        "Sandbox",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
            next_state.set(GameState::Sandbox);
        },
    );

    ui.add_button_observe(
        "Load Game",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut load_writer: MessageWriter<LoadGameRequest>| {
            load_writer.write(LoadGameRequest);
        },
    );

    ui.build();
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Pause Menu (overlay — GameState stays Playing)
// ============================================================================

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    paused: Option<Res<GamePaused>>,
    theme: Res<UiTheme>,
    pause_menu: Query<Entity, With<PauseMenu>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if paused.is_some() {
        // Unpause: remove resource and despawn overlay
        commands.remove_resource::<GamePaused>();
        for entity in pause_menu.iter() {
            commands.entity(entity).despawn();
        }
    } else {
        // Pause: insert resource and spawn overlay
        commands.insert_resource(GamePaused);
        spawn_pause_menu(commands, &theme);
    }
}

fn spawn_pause_menu(commands: Commands, _theme: &UiTheme) {
    let mut ui = UIBuilder::new(commands, None);

    ui.component::<PauseMenu>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .gap_px(16.0);

    ui.add_text_child("Paused", None, Some(48.0), None);

    ui.add_button_observe(
        "Resume",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>,
         mut commands: Commands,
         pause_menu: Query<Entity, With<PauseMenu>>| {
            info!("Resume button clicked!");
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
        },
    );

    ui.add_button_observe(
        "Save Game",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut save_writer: MessageWriter<SaveGameRequest>| {
            save_writer.write(SaveGameRequest);
        },
    );

    ui.add_button_observe(
        "Load Game",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut load_writer: MessageWriter<LoadGameRequest>| {
            load_writer.write(LoadGameRequest);
        },
    );

    ui.add_button_observe(
        "Main Menu",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>,
         mut commands: Commands,
         mut next_state: ResMut<NextState<GameState>>,
         pause_menu: Query<Entity, With<PauseMenu>>| {
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
            next_state.set(GameState::Menu);
        },
    );

    ui.build();
}
