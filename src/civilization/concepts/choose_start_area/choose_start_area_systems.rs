use crate::GameActivity;
use crate::civilization::DebugOptions;
use crate::civilization::components::GameCamera;
use crate::civilization::MoveTokensFromStockToAreaCommand;
use crate::civilization::concepts::choose_start_area::choose_start_area_components::*;
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::stupid_ai::IsHuman;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const CLICK_RADIUS: f32 = 30.0;

/// Drives start-area selection to completion and owns `StartGame`'s exit,
/// mirroring the calamity/monotheism selection systems: a human player with
/// more than one eligible start area gets `AwaitingStartAreaSelection` and a
/// click on the map resolves it; this system applies the result once that
/// marker is cleared.
pub fn apply_start_area_choice(
    mut commands: Commands,
    needing: Query<(Entity, &NeedsToChooseStartArea)>,
    awaiting: Query<Entity, With<AwaitingStartAreaSelection>>,
    mut selection_state: ResMut<StartAreaSelectionState>,
    mut writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>,
    debug_options: Res<DebugOptions>,
    loading_from_save: Option<Res<LoadingFromSave>>,
) {
    // When resuming from a save, `start_game` (OnEnter(StartGame)) already
    // queued the correct saved activity via `NextState`. No player gets
    // `NeedsToChooseStartArea` on a resumed game, so `needing`/`awaiting`
    // are both empty this frame too -- without this guard this system's
    // "nothing to choose" branch runs later in the schedule and clobbers
    // that queued transition with `PopulationExpansion`, re-running
    // expansion (and duplicating tokens) on top of the restored board.
    if needing.is_empty() && awaiting.is_empty() {
        if loading_from_save.is_some() {
            return;
        }
        let start_activity = debug_options
            .start_at_activity
            .clone()
            .unwrap_or(GameActivity::PopulationExpansion);
        next_state.set(start_activity);
        return;
    }

    for (player_entity, needs) in &needing {
        let is_waiting = awaiting.get(player_entity).is_ok();
        if is_waiting {
            continue;
        }
        if selection_state.player == Some(player_entity) {
            if let Some(chosen) = selection_state.chosen {
                writer.write(MoveTokensFromStockToAreaCommand {
                    area_entity: chosen,
                    player_entity,
                    number_of_tokens: 1,
                });
                info!(
                    "[START AREA] Human player {:?} chose area {:?}",
                    player_entity, chosen
                );
            }
            commands
                .entity(player_entity)
                .remove::<NeedsToChooseStartArea>();
            *selection_state = StartAreaSelectionState::default();
        } else if selection_state.player.is_none() {
            selection_state.player = Some(player_entity);
            selection_state.eligible.clone_from(&needs.eligible);
            selection_state.chosen = None;
            commands
                .entity(player_entity)
                .insert(AwaitingStartAreaSelection);
        }
    }
}

pub fn draw_start_area_choice_gizmos(
    mut gizmos: Gizmos,
    state: Res<StartAreaSelectionState>,
    transforms: Query<&Transform>,
) {
    for &area in &state.eligible {
        if let Ok(t) = transforms.get(area) {
            gizmos.circle_2d(t.translation.truncate(), 22.0, Color::srgb(0.2, 1.0, 0.3));
        }
    }
}

/// A single click on one of the highlighted eligible areas both chooses and
/// confirms it -- no separate confirm step needed.
pub fn handle_start_area_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    transforms: Query<&Transform>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingStartAreaSelection>)>,
    mut state: ResMut<StartAreaSelectionState>,
    mut commands: Commands,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(player) = human_waiting.single() else {
        return;
    };
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let clicked = state
        .eligible
        .iter()
        .copied()
        .filter_map(|e| {
            transforms
                .get(e)
                .ok()
                .map(|t| (e, world_pos.distance(t.translation.truncate())))
        })
        .filter(|&(_, d)| d <= CLICK_RADIUS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, _)| e);

    if let Some(area) = clicked {
        state.chosen = Some(area);
        commands
            .entity(player)
            .remove::<AwaitingStartAreaSelection>();
    }
}
