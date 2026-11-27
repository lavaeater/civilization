//! What the human sees and clicks on the board while moving: fat arrows
//! from the focused area to everywhere it can go, overland and by sea at the
//! same time, with how many can go written on each.
//!
//! Lives in its own plugin because it needs a window, a camera and the gizmo
//! renderer; the headless server only has the rules (`MovementPlugin`).

use crate::GameActivity;
use crate::civilization::components::{GameArea, GameCamera};
use crate::civilization::concepts::movement::movement_components::PerformingMovement;
use crate::civilization::concepts::movement::movement_ui_components::MovementSelectionState;
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::stupid_ai::IsHuman;
use bevy::color::Alpha;
use bevy::gizmos::AppGizmoBuilder;
use bevy::gizmos::config::{
    GizmoConfig, GizmoConfigGroup, GizmoLineConfig, GizmoLineJoint, GizmoLineStyle,
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use bevy::ui_widgets::Button as WidgetsButton;
use bevy::window::PrimaryWindow;

/// How close (world units) a click must land to an area's centre to pick it.
const CLICK_RADIUS: f32 = 30.0;
/// Arrows stop short of the centres so the tip and the tokens stay readable.
const ARROW_GAP: f32 = 16.0;
/// How far a sea route bows out from the straight line, per unit of length.
const SEA_BEND: f32 = 0.3;
const LABEL_Z: f32 = 30.0;

/// Solid, fat lines: land arrows, arrowheads and rings.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MoveArrowGizmos;

/// Dashed lines: a ship's course.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct SeaRouteGizmos;

/// A world-space "×4" written on an arrow.
#[derive(Component)]
pub struct MovementArrowLabel;

pub struct MovementBoardPlugin;

impl Plugin for MovementBoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_gizmo_config(
            MoveArrowGizmos,
            GizmoConfig {
                line: GizmoLineConfig {
                    width: 7.0,
                    joints: GizmoLineJoint::Round(4),
                    ..default()
                },
                ..default()
            },
        )
        .insert_gizmo_config(
            SeaRouteGizmos,
            GizmoConfig {
                line: GizmoLineConfig {
                    width: 5.0,
                    style: GizmoLineStyle::Dashed {
                        gap_scale: 1.4,
                        line_scale: 2.5,
                    },
                    ..default()
                },
                ..default()
            },
        )
        .add_systems(
            Update,
            (
                handle_movement_target_click,
                draw_movement_arrows,
                update_movement_labels,
            )
                .chain()
                .run_if(in_state(GameActivity::Movement)),
        )
        .add_systems(OnExit(GameActivity::Movement), despawn_movement_labels);
    }
}

/// How a target is reached from the focused area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    Walk,
    Attack,
    CityAttack,
    Sail,
}

/// One place the focused area can send tokens to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetOption {
    pub target: Entity,
    pub kind: TargetKind,
    /// Walking: how many may go. Sailing: how many tokens can be aboard when
    /// the ship leaves (those already aboard plus those who can step on).
    pub max: usize,
}

impl TargetOption {
    pub fn is_attack(self) -> bool {
        matches!(self.kind, TargetKind::Attack | TargetKind::CityAttack)
    }

    fn label(self) -> String {
        match self.kind {
            TargetKind::Walk => format!("×{}", self.max),
            TargetKind::Attack => format!("attack ×{}", self.max),
            TargetKind::CityAttack => format!("city! ×{}", self.max),
            TargetKind::Sail => format!("ship ≤{}", self.max),
        }
    }

    fn color(self) -> Color {
        match self.kind {
            TargetKind::Walk => Color::srgb(0.25, 0.65, 1.0),
            TargetKind::Attack => Color::srgb(1.0, 0.6, 0.15),
            TargetKind::CityAttack => Color::srgb(1.0, 0.2, 0.2),
            TargetKind::Sail => Color::srgb(0.1, 0.95, 0.95),
        }
    }
}

/// Everywhere `source` can go, sorted so land comes before sea for a target
/// reachable both ways. `ship_cargo` is how many are aboard the ship there.
pub fn targets_from<'a>(
    moves: impl Iterator<Item = &'a GameMove>,
    source: Entity,
    ship_cargo: usize,
) -> Vec<TargetOption> {
    let mut options: Vec<TargetOption> = moves
        .filter_map(|game_move| {
            let (m, kind) = match game_move {
                GameMove::Movement(m) => (m, TargetKind::Walk),
                GameMove::AttackArea(m) => (m, TargetKind::Attack),
                GameMove::AttackCity(m) => (m, TargetKind::CityAttack),
                GameMove::ShipFerry(m) => (m, TargetKind::Sail),
                _ => return None,
            };
            (m.source == source).then(|| TargetOption {
                target: m.target,
                kind,
                max: if kind == TargetKind::Sail {
                    ship_cargo + m.max_tokens
                } else {
                    m.max_tokens
                },
            })
        })
        .collect();
    // `AvailableMoves` is a HashMap: give the board a stable order.
    options.sort_by_key(|o| (o.target, o.kind == TargetKind::Sail));
    options.dedup_by_key(|o| (o.target, o.kind == TargetKind::Sail));
    options
}

/// Which option a click on `target` means: overland if it can be walked to,
/// unless the player holds Shift (or the sea is the only way there).
pub fn option_for_click(
    options: &[TargetOption],
    target: Entity,
    prefer_sea: bool,
) -> Option<TargetOption> {
    let land = options
        .iter()
        .find(|o| o.target == target && o.kind != TargetKind::Sail);
    let sea = options
        .iter()
        .find(|o| o.target == target && o.kind == TargetKind::Sail);
    if prefer_sea {
        sea.or(land).copied()
    } else {
        land.or(sea).copied()
    }
}

fn focused_options(
    humans: &Query<&AvailableMoves, (With<IsHuman>, With<PerformingMovement>)>,
    selection: &MovementSelectionState,
) -> Vec<TargetOption> {
    let Some(source) = selection.current_source() else {
        return Vec::new();
    };
    humans
        .iter()
        .flat_map(|moves| targets_from(moves.moves.values(), source, selection.ship_cargo))
        .collect()
}

fn cursor_world(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera: &Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) -> Option<Vec2> {
    let cursor = windows.single().ok()?.cursor_position()?;
    let (camera, camera_transform) = camera.single().ok()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

/// The target under the cursor, if any.
fn hovered_target(
    options: &[TargetOption],
    cursor: Option<Vec2>,
    areas: &Query<&Transform, With<GameArea>>,
) -> Option<Entity> {
    let cursor = cursor?;
    options
        .iter()
        .filter_map(|o| {
            let pos = areas.get(o.target).ok()?.translation.truncate();
            let d = pos.distance(cursor);
            (d <= CLICK_RADIUS).then_some((o.target, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(target, _)| target)
}

/// Points along the route from `from` to `to`, trimmed at both ends. Sea
/// routes bow out to one side so they don't hide a land arrow to the same
/// place.
fn route_points(from: Vec2, to: Vec2, sail: bool) -> Vec<Vec2> {
    let delta = to - from;
    let length = delta.length();
    if length <= ARROW_GAP * 2.0 {
        return vec![from, to];
    }
    if !sail {
        let dir = delta / length;
        return vec![from + dir * ARROW_GAP, to - dir * ARROW_GAP];
    }
    let control = (from + to) / 2.0 + delta.perp().normalize() * length * SEA_BEND;
    let bezier = |t: f32| from.lerp(control, t).lerp(control.lerp(to, t), t);
    // Trim by parameter: close enough to the gap for a gentle bow.
    let t0 = ARROW_GAP / length;
    let steps = 24;
    (0..=steps)
        .map(|i| bezier(t0 + (1.0 - 2.0 * t0) * i as f32 / steps as f32))
        .collect()
}

fn arrowhead(gizmos: &mut Gizmos<MoveArrowGizmos>, tip: Vec2, from: Vec2, color: Color) {
    let dir = (tip - from).normalize_or_zero();
    let back = tip - dir * 16.0;
    let side = dir.perp() * 10.0;
    gizmos.linestrip_2d([back + side, tip, back - side], color);
}

/// Fat arrows from the focused area to each place it can go; once one is
/// picked, only that one, pulsing.
pub fn draw_movement_arrows(
    mut arrows: Gizmos<MoveArrowGizmos>,
    mut sea: Gizmos<SeaRouteGizmos>,
    mut thin: Gizmos,
    humans: Query<&AvailableMoves, (With<IsHuman>, With<PerformingMovement>)>,
    areas: Query<&Transform, With<GameArea>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    selection: Res<MovementSelectionState>,
    time: Res<Time>,
) {
    let Some(source) = selection.current_source() else {
        return;
    };
    let Ok(source_pos) = areas.get(source).map(|t| t.translation.truncate()) else {
        return;
    };
    let options = focused_options(&humans, &selection);
    if options.is_empty() {
        return;
    }
    // Where we're moving from.
    thin.circle_2d(source_pos, 24.0, Color::srgb(1.0, 0.9, 0.2));
    thin.circle_2d(source_pos, 26.0, Color::srgb(1.0, 0.9, 0.2));

    let picked = selection
        .target_area
        .filter(|_| selection.has_selection() && selection.source_area == Some(source));
    let hovered = hovered_target(&options, cursor_world(&windows, &camera), &areas);
    let pulse = 0.65 + 0.35 * (time.elapsed_secs() * 5.0).sin();

    for option in &options {
        let sail = option.kind == TargetKind::Sail;
        if let Some(picked) = picked
            && (option.target != picked || sail != selection.by_ship)
        {
            continue;
        }
        let Ok(target_pos) = areas.get(option.target).map(|t| t.translation.truncate()) else {
            continue;
        };
        let color = if picked.is_some() {
            Color::srgb(0.25, 1.0, 0.3).with_alpha(pulse)
        } else if hovered == Some(option.target) {
            option.color().mix(&Color::WHITE, 0.45)
        } else {
            option.color()
        };
        let points = route_points(source_pos, target_pos, sail);
        if sail {
            sea.linestrip_2d(points.iter().copied(), color);
        } else {
            arrows.linestrip_2d(points.iter().copied(), color);
        }
        if let [.., before, tip] = points[..] {
            arrowhead(&mut arrows, tip, before, color);
        }
        thin.circle_2d(target_pos, 20.0, color);
        if hovered == Some(option.target) {
            thin.circle_2d(target_pos, 23.0, Color::WHITE);
        }
    }
}

/// Click an arrow's target to pick it. Shift-click sails where land also goes.
pub fn handle_movement_target_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    humans: Query<&AvailableMoves, (With<IsHuman>, With<PerformingMovement>)>,
    human_entity: Query<Entity, (With<IsHuman>, With<PerformingMovement>)>,
    areas: Query<&Transform, With<GameArea>>,
    ui_button_hover: Query<&Hovered, With<WidgetsButton>>,
    mut selection: ResMut<MovementSelectionState>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    // The controls panel sits over the board; a click on its buttons must
    // not also pick whatever area is underneath.
    if ui_button_hover.iter().any(|hovered| hovered.0) {
        return;
    }
    let (Some(source), Ok(player)) = (selection.current_source(), human_entity.single()) else {
        return;
    };
    let options = focused_options(&humans, &selection);
    let Some(target) = hovered_target(&options, cursor_world(&windows, &camera), &areas) else {
        return;
    };
    let prefer_sea = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if let Some(option) = option_for_click(&options, target, prefer_sea) {
        selection.select_target(player, source, option);
    }
}

/// Writes how many can go on each arrow. Rebuilt only when the choices or the
/// selection change.
pub fn update_movement_labels(
    mut commands: Commands,
    humans: Query<&AvailableMoves, (With<IsHuman>, With<PerformingMovement>)>,
    changed: Query<(), (With<IsHuman>, Changed<AvailableMoves>)>,
    areas: Query<&Transform, With<GameArea>>,
    labels: Query<Entity, With<MovementArrowLabel>>,
    selection: Res<MovementSelectionState>,
    asset_server: Res<AssetServer>,
    mut had_movers: Local<bool>,
) {
    let movers = !humans.is_empty();
    if !selection.is_changed() && changed.is_empty() && movers == *had_movers {
        return;
    }
    *had_movers = movers;
    for label in &labels {
        commands.entity(label).despawn();
    }
    let Some(source) = selection.current_source() else {
        return;
    };
    let Ok(source_pos) = areas.get(source).map(|t| t.translation.truncate()) else {
        return;
    };
    let picked = selection
        .target_area
        .filter(|_| selection.has_selection() && selection.source_area == Some(source));
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    for option in focused_options(&humans, &selection) {
        let sail = option.kind == TargetKind::Sail;
        if picked.is_some_and(|p| p != option.target || sail != selection.by_ship) {
            continue;
        }
        let Ok(target_pos) = areas.get(option.target).map(|t| t.translation.truncate()) else {
            continue;
        };
        let points = route_points(source_pos, target_pos, sail);
        // Sea labels sit on the bow; land labels near the tip.
        let at = if sail {
            points[points.len() / 2]
        } else {
            points[0].lerp(points[points.len() - 1], 0.7)
        };
        let text = if picked.is_some() {
            if sail {
                format!("{} aboard", selection.token_count)
            } else {
                format!("{} going", selection.token_count)
            }
        } else {
            option.label()
        };
        commands.spawn((
            MovementArrowLabel,
            Text2d::new(text),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(option.color().mix(&Color::WHITE, 0.6)),
            Text2dShadow {
                offset: Vec2::new(1.5, -1.5),
                color: Color::BLACK,
            },
            Transform::from_translation(at.extend(LABEL_Z)),
        ));
    }
}

pub fn despawn_movement_labels(
    mut commands: Commands,
    labels: Query<Entity, With<MovementArrowLabel>>,
) {
    for label in &labels {
        commands.entity(label).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::game_moves::MovementMove;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    fn moves(source: Entity) -> Vec<GameMove> {
        let player = e(99);
        vec![
            GameMove::Movement(MovementMove::new(source, e(2), player, 3)),
            GameMove::ShipFerry(MovementMove::new(source, e(2), player, 2)),
            GameMove::ShipFerry(MovementMove::new(source, e(3), player, 2)),
            GameMove::AttackArea(MovementMove::new(source, e(4), player, 3)),
            GameMove::Movement(MovementMove::new(e(7), e(8), player, 1)),
            GameMove::EndMovement,
        ]
    }

    /// Land and sea come together; only the focused area's, and a ship
    /// counts those already aboard.
    #[test]
    fn land_and_sea_targets_show_together() {
        let all = moves(e(1));
        let mut options = targets_from(all.iter(), e(1), 1);
        options.sort_by_key(|o| (o.target.index(), o.kind == TargetKind::Sail));
        assert_eq!(
            options,
            vec![
                TargetOption {
                    target: e(2),
                    kind: TargetKind::Walk,
                    max: 3
                },
                TargetOption {
                    target: e(2),
                    kind: TargetKind::Sail,
                    max: 3
                },
                TargetOption {
                    target: e(3),
                    kind: TargetKind::Sail,
                    max: 3
                },
                TargetOption {
                    target: e(4),
                    kind: TargetKind::Attack,
                    max: 3
                },
            ]
        );
    }

    /// A plain click walks when it can; Shift sails; sea-only targets sail
    /// either way.
    #[test]
    fn a_click_walks_unless_shift_or_sea_only() {
        let all = moves(e(1));
        let options = targets_from(all.iter(), e(1), 0);
        let kind = |target, shift| option_for_click(&options, target, shift).map(|o| o.kind);
        assert_eq!(kind(e(2), false), Some(TargetKind::Walk));
        assert_eq!(kind(e(2), true), Some(TargetKind::Sail));
        assert_eq!(kind(e(3), false), Some(TargetKind::Sail));
        assert_eq!(kind(e(4), true), Some(TargetKind::Attack));
        assert_eq!(kind(e(8), false), None, "not from the focused area");
    }

    #[test]
    fn sea_routes_bow_away_from_the_straight_line() {
        let land = route_points(Vec2::ZERO, Vec2::new(200.0, 0.0), false);
        let sea = route_points(Vec2::ZERO, Vec2::new(200.0, 0.0), true);
        assert_eq!(land.len(), 2);
        assert!(land.iter().all(|p| p.y == 0.0));
        assert!(sea[sea.len() / 2].y.abs() > 20.0, "visibly curved");
        assert!(sea[0].length() > 5.0, "starts clear of the source");
    }
}
