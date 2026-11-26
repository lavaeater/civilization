//! Replays the [`NarrationQueue`] as toasts, one at a time, and walks the
//! camera to where each thing happened.

use super::chronicle_core::{Narration, NarrationQueue};
use crate::civilization::DebugOptions;
use crate::civilization::concepts::map::CameraFocusQueue;
use bevy::prelude::*;

/// How long a toast stays up, shorter when many are waiting so the replay
/// keeps up with the game.
fn display_secs(waiting: usize) -> f32 {
    match waiting {
        0..=2 => 3.5,
        3..=6 => 2.5,
        _ => 1.5,
    }
}

#[derive(Resource, Default)]
pub struct Narrator {
    showing: Option<Timer>,
}

#[derive(Component)]
pub struct NarrationToast;

pub fn play_narration(
    time: Res<Time>,
    mut narrator: ResMut<Narrator>,
    mut queue: ResMut<NarrationQueue>,
    toasts: Query<Entity, With<NarrationToast>>,
    areas: Query<&Transform>,
    mut camera: ResMut<CameraFocusQueue>,
    debug: Option<Res<DebugOptions>>,
    mut commands: Commands,
) {
    if let Some(timer) = narrator.showing.as_mut() {
        timer.tick(time.delta());
        if !timer.is_finished() {
            return;
        }
        narrator.showing = None;
        for toast in &toasts {
            commands.entity(toast).despawn();
        }
    }
    let Some(next) = queue.0.pop_front() else {
        return;
    };
    let secs = display_secs(queue.0.len());
    let auto_pan = debug.is_none_or(|d| !d.static_map_view);
    if auto_pan
        && let Some(area) = next.entry.area
        && let Ok(transform) = areas.get(area)
    {
        camera.add_focus(transform.translation, 0.0, next.entry.text.clone());
    }
    spawn_toast(&mut commands, &next);
    narrator.showing = Some(Timer::from_seconds(secs, TimerMode::Once));
}

fn spawn_toast(commands: &mut Commands, narration: &Narration) {
    commands
        .spawn((
            NarrationToast,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(50),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    max_width: Val::Px(640.0),
                    padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                    border: UiRect::left(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.07, 0.06, 0.92)),
                BorderColor::all(Color::srgb(0.85, 0.65, 0.25)),
                Pickable::IGNORE,
            ))
            .with_children(|toast| {
                toast.spawn((
                    Text::new(narration.when.clone()),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.7, 0.65, 0.55)),
                    Pickable::IGNORE,
                ));
                toast.spawn((
                    Text::new(narration.entry.text.clone()),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::srgb(0.95, 0.92, 0.85)),
                    Pickable::IGNORE,
                ));
            });
        });
}

/// Leaving the game drops whatever was still to be told.
pub fn clear_narration(
    mut narrator: ResMut<Narrator>,
    mut queue: ResMut<NarrationQueue>,
    toasts: Query<Entity, With<NarrationToast>>,
    mut commands: Commands,
) {
    narrator.showing = None;
    queue.0.clear();
    for toast in &toasts {
        commands.entity(toast).despawn();
    }
}
