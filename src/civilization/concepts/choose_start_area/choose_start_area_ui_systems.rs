use bevy::prelude::*;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

use crate::civilization::Z_DIALOG;
use crate::civilization::concepts::choose_start_area::choose_start_area_components::*;
use crate::stupid_ai::IsHuman;

#[derive(Component, Default)]
pub struct StartAreaBannerRoot;

pub fn spawn_start_area_banner(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingStartAreaSelection>)>,
    existing: Query<Entity, With<StartAreaBannerRoot>>,
    commands: Commands,
    theme: Res<LavaTheme>,
) {
    if !existing.is_empty() || human_waiting.iter().next().is_none() {
        return;
    }

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<StartAreaBannerRoot>()
        .absolute_position()
        .top(Val::Px(20.0))
        .left(Val::Percent(50.0))
        .display_flex()
        .flex_column()
        .align_items_center()
        .padding_all_px(10.0)
        .z_index(Z_DIALOG)
        .bg_color(Color::srgba(0.05, 0.1, 0.15, 0.93));

    ui.add_text_child(
        "Choose your starting area -- click one of the highlighted areas",
        Some(TextStyle::size_color(14.0, Color::srgb(0.4, 1.0, 0.5))),
    );

    ui.build();
}

pub fn cleanup_start_area_banner(
    mut commands: Commands,
    root: Query<Entity, With<StartAreaBannerRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingStartAreaSelection>)>,
) {
    if !root.is_empty() && human_waiting.is_empty() {
        for entity in &root {
            commands.entity(entity).despawn();
        }
    }
}

/// Safety net: force-despawn the banner and reset selection state on
/// `StartGame`'s exit, in case it ends while the banner is still up.
pub fn cleanup_start_area_banner_on_exit(
    mut commands: Commands,
    root: Query<Entity, With<StartAreaBannerRoot>>,
    mut state: ResMut<StartAreaSelectionState>,
) {
    for entity in &root {
        commands.entity(entity).despawn();
    }
    *state = StartAreaSelectionState::default();
}
