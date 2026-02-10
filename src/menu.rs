use crate::civilization::components::GameCamera;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use lava_ui_builder::{UIBuilder, UiTheme};
use moonshine_save::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                handle_menu_buttons.run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnExit(GameState::Menu), cleanup_menu)
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(
                Update,
                (
                    handle_pause_buttons.run_if(in_state(GameState::Paused)),
                    toggle_pause.run_if(in_state(GameState::Paused)),
                ),
            )
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
            .add_systems(
                Update,
                toggle_pause_from_playing.run_if(in_state(GameState::Playing)),
            );
    }
}

// ============================================================================
// Shared components
// ============================================================================

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct LoadGameButton;

#[derive(Component)]
struct SaveGameButton;

#[derive(Component, Default)]
struct Menu;

#[derive(Component, Default)]
struct PauseMenu;

// ============================================================================
// Main Menu
// ============================================================================

fn setup_menu(
    mut commands: Commands,
    _textures: Res<TextureAssets>,
    theme: Res<UiTheme>,
) {
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection::default_2d()),
        GameCamera,
        Msaa::Off,
    ));

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.with_component::<Menu>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .gap_px(16.0);

    ui.add_text_child("Advanced Civilization", None, Some(48.0), None);

    ui.add_themed_button(ChangeState(GameState::Playing), |btn| {
        btn.text("Play").size_px(200.0, 60.0);
    });

    ui.add_themed_button(ChangeState(GameState::Sandbox), |btn| {
        btn.text("Sandbox").size_px(200.0, 60.0);
    });

    ui.add_themed_button(LoadGameButton, |btn| {
        btn.text("Load Game").size_px(200.0, 60.0);
    });

    ui.build();
}

fn handle_menu_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    change_query: Query<(&Interaction, &ChangeState), (Changed<Interaction>, With<Button>)>,
    load_query: Query<&Interaction, (Changed<Interaction>, With<LoadGameButton>, With<Button>)>,
) {
    for (interaction, change_state) in &change_query {
        if *interaction == Interaction::Pressed {
            next_state.set(change_state.0.clone());
        }
    }
    for interaction in &load_query {
        if *interaction == Interaction::Pressed {
            info!("Loading game from main menu...");
            commands.trigger_load(LoadWorld::default_from_file("savegame.ron"));
            next_state.set(GameState::Playing);
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Pause Menu
// ============================================================================

fn toggle_pause_from_playing(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }
}

fn setup_pause_menu(commands: Commands, theme: Res<UiTheme>) {
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.with_component::<PauseMenu>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.7))
        .gap_px(16.0);

    ui.add_text_child("Paused", None, Some(48.0), None);

    ui.add_themed_button(ChangeState(GameState::Playing), |btn| {
        btn.text("Resume").size_px(200.0, 60.0);
    });

    ui.add_themed_button(SaveGameButton, |btn| {
        btn.text("Save Game").size_px(200.0, 60.0);
    });

    ui.add_themed_button(LoadGameButton, |btn| {
        btn.text("Load Game").size_px(200.0, 60.0);
    });

    ui.add_themed_button(ChangeState(GameState::Menu), |btn| {
        btn.text("Main Menu").size_px(200.0, 60.0);
    });

    ui.build();
}

fn handle_pause_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    change_query: Query<(&Interaction, &ChangeState), (Changed<Interaction>, With<Button>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<SaveGameButton>, With<Button>)>,
    load_query: Query<&Interaction, (Changed<Interaction>, With<LoadGameButton>, With<Button>)>,
) {
    for (interaction, change_state) in &change_query {
        if *interaction == Interaction::Pressed {
            next_state.set(change_state.0.clone());
        }
    }
    for interaction in &save_query {
        if *interaction == Interaction::Pressed {
            info!("Saving game...");
            commands.trigger_save(SaveWorld::default_into_file("savegame.ron"));
        }
    }
    for interaction in &load_query {
        if *interaction == Interaction::Pressed {
            info!("Loading game...");
            commands.trigger_load(LoadWorld::default_from_file("savegame.ron"));
            next_state.set(GameState::Playing);
        }
    }
}

fn cleanup_pause_menu(mut commands: Commands, menu: Query<Entity, With<PauseMenu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}