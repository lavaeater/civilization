use crate::civilization::components::GameCamera;
use crate::loading::TextureAssets;
use crate::{GamePaused, GameState};
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
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::Playing)),
                    handle_pause_buttons.run_if(resource_exists::<GamePaused>),
                ),
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

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct MainMenuButton;

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
        btn.text("Play").size_px(300.0, 60.0);
    });

    ui.add_themed_button(ChangeState(GameState::Sandbox), |btn| {
        btn.text("Sandbox").size_px(300.0, 60.0);
    });

    ui.add_themed_button(LoadGameButton, |btn| {
        btn.text("Load Game").size_px(300.0, 60.0);
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

fn spawn_pause_menu(commands: Commands, theme: &UiTheme) {
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

    ui.add_themed_button(ResumeButton, |btn| {
        btn.text("Resume").size_px(300.0, 60.0);
    });

    ui.add_themed_button(SaveGameButton, |btn| {
        btn.text("Save Game").size_px(300.0, 60.0);
    });

    ui.add_themed_button(LoadGameButton, |btn| {
        btn.text("Load Game").size_px(300.0, 60.0);
    });

    ui.add_themed_button(MainMenuButton, |btn| {
        btn.text("Main Menu").size_px(300.0, 60.0);
    });

    ui.build();
}

fn handle_pause_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    resume_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>, With<Button>)>,
    save_query: Query<&Interaction, (Changed<Interaction>, With<SaveGameButton>, With<Button>)>,
    load_query: Query<&Interaction, (Changed<Interaction>, With<LoadGameButton>, With<Button>)>,
    main_menu_query: Query<&Interaction, (Changed<Interaction>, With<MainMenuButton>, With<Button>)>,
    pause_menu: Query<Entity, With<PauseMenu>>,
) {
    for interaction in &resume_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
            return;
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
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
            commands.trigger_load(LoadWorld::default_from_file("savegame.ron"));
        }
    }
    for interaction in &main_menu_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<GamePaused>();
            for entity in pause_menu.iter() {
                commands.entity(entity).despawn();
            }
            next_state.set(GameState::Menu);
        }
    }
}