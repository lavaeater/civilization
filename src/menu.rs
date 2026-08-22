use crate::civilization::save_game::{LoadGameRequest, SaveGameRequest};
use crate::civilization::{
    CityToken, CivilizationTradeCards, GameArea, GameCamera, GameInfoAndStuff, GameResult, Token,
};
use crate::loading::TextureAssets;
use crate::player::Player;
use crate::{GameActivity, GamePaused, GameState};
use bevy::feathers::FeathersPlugins;
use bevy::{
    feathers::{dark_theme::create_dark_theme, theme::UiTheme},
    prelude::*,
    ui_widgets::Activate,
};
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

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
                spawn_victory_screen.run_if(in_state(GameActivity::GameOver)),
            )
            .add_systems(Update, (toggle_pause.run_if(in_state(GameState::Playing)),));
    }
}

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
    theme: Res<LavaTheme>,
    existing_camera: Query<(), With<GameCamera>>,
) {
    // Re-entering the menu (e.g. after leaving an online game) must not
    // spawn a second camera.
    if existing_camera.is_empty() {
        commands.spawn((
            Camera2d,
            IsDefaultUiCamera,
            Projection::Orthographic(OrthographicProjection::default_2d()),
            GameCamera,
            Msaa::Off,
        ));
    }

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<Menu>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .gap_px(16.0);

    ui.add_text_child("Advanced Civilization", Some(TextStyle::size(48.0)));

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
        "Play Online",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
            next_state.set(GameState::Online);
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

    ui.add_button_observe(
        "Map Editor",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
            next_state.set(GameState::MapEditor);
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
// Victory Screen (rule 35 — shown on GameActivity::GameOver)
// ============================================================================

#[derive(Component, Default)]
struct VictoryScreen;

/// All game-world entity roots to tear down for a fresh game. UI children carry
/// `ChildOf` and are despawned recursively via their root, so we only take roots.
/// The camera is handled separately (kept for New Game, despawned for Main Menu).
type GameWorldRoots = (
    Without<ChildOf>,
    Or<(
        With<Player>,
        With<Token>,
        With<CityToken>,
        With<GameArea>,
        With<Node>,
    )>,
);

fn reset_game_resources(commands: &mut Commands) {
    // A fresh game needs a full trade-card deck and a reset round counter. The
    // map/faction resources are rebuilt by `load_map` on entering PrepareGame.
    commands.insert_resource(CivilizationTradeCards::new());
    commands.insert_resource(GameInfoAndStuff::default());
    // Drop the finished game's standings so the next game's victory screen can't
    // read stale results before `determine_winner` republishes them.
    commands.remove_resource::<GameResult>();
}

fn spawn_victory_screen(
    commands: Commands,
    result: Option<Res<GameResult>>,
    existing: Query<(), With<VictoryScreen>>,
    theme: Res<LavaTheme>,
) {
    // Spawn exactly once, and only after `determine_winner` has published the
    // standings (it inserts GameResult via a deferred command on the same frame
    // the GameOver state is entered).
    if !existing.is_empty() {
        return;
    }
    let Some(result) = result else {
        return;
    };

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<VictoryScreen>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .gap_px(10.0);

    ui.add_text_child("Game Over", Some(TextStyle::size(48.0)));

    if let Some((winner, score, _)) = result.standings.first() {
        ui.add_text_child(
            format!("🏆 {winner} wins with {score} points"),
            Some(TextStyle::size(28.0)),
        );
    }
    for (i, (name, score, space)) in result.standings.iter().enumerate() {
        ui.add_text_child(
            format!("{}. {} — {} pts (A.S.T. {})", i + 1, name, score, space),
            Some(TextStyle::size(18.0)),
        );
    }

    ui.add_button_observe(
        "New Game",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>,
         mut commands: Commands,
         roots: Query<Entity, GameWorldRoots>,
         mut next_state: ResMut<NextState<GameActivity>>| {
            // Tear down the finished game (keep the camera — load_map reuses it)
            // and restart by re-entering the setup phase.
            for entity in roots.iter() {
                commands.entity(entity).despawn();
            }
            reset_game_resources(&mut commands);
            next_state.set(GameActivity::PrepareGame);
        },
    );

    ui.add_button_observe(
        "Main Menu",
        |btn| {
            btn.size(px(300.0), px(60.0));
        },
        |_activate: On<Activate>,
         mut commands: Commands,
         roots: Query<Entity, GameWorldRoots>,
         camera: Query<Entity, With<GameCamera>>,
         mut next_state: ResMut<NextState<GameState>>| {
            for entity in roots.iter() {
                commands.entity(entity).despawn();
            }
            // The main menu spawns its own camera, so drop this one too.
            for entity in camera.iter() {
                commands.entity(entity).despawn();
            }
            reset_game_resources(&mut commands);
            next_state.set(GameState::Menu);
        },
    );

    ui.build();
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

    ui.add_text_child("Paused", Some(TextStyle::size(48.0)));

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
