use bevy::prelude::*;
use bevy::ui::ZIndex;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

use crate::civilization::Z_PANEL;
use crate::civilization::components::Faction;
use crate::civilization::concepts::succession::succession_components::{
    AstEpoch, AstPosition, AstTrack,
};
use crate::civilization::enums::GameFaction;
use crate::player::Player;

/// Root node of the A.S.T. panel.
#[derive(Component)]
pub struct AstUiRoot;

/// The relatively-positioned row that holds the 17 track cells; markers are
/// absolutely positioned within it.
#[derive(Component)]
pub struct AstTrackRow;

/// One space on the rendered track.
#[derive(Component)]
pub struct AstCell {
    pub index: u32,
}

/// A faction marker chip; positioned over its player's current space each frame.
#[derive(Component)]
pub struct AstMarker {
    pub player: Entity,
}

// ── Geometry ─────────────────────────────────────────────────────────────────
const CELL_W: f32 = 36.0;
const CELL_H: f32 = 34.0;
const MARKER: f32 = 15.0;
/// Gap between adjacent markers sharing a space, in either axis.
const MARKER_GAP: f32 = 3.0;
/// Markers sharing a space arrange into a grid this many columns wide, rather
/// than one tall single-file column -- with up to 9 factions all starting on
/// space 0, a single column would overlap badly (the old fixed 9px vertical
/// step was smaller than the 15px marker itself) or run far off the bottom of
/// the panel.
const STACK_COLS: usize = 2;
/// Centre-to-centre step between grid cells (marker size + gap).
const STACK_STEP: f32 = MARKER + MARKER_GAP;

/// Authentic civilization colour from `docs/ast.xslx` (sheet "AST").
pub fn ast_faction_color(faction: GameFaction) -> Color {
    match faction {
        GameFaction::Africa => Color::srgb(0.60, 0.20, 0.00),
        GameFaction::Iberia => Color::srgb(1.00, 0.00, 0.00),
        GameFaction::Illyria => Color::srgb(1.00, 1.00, 0.00),
        GameFaction::Thrace => Color::srgb(0.00, 0.50, 0.00),
        GameFaction::Crete => Color::srgb(0.60, 0.80, 0.00),
        GameFaction::Asia => Color::srgb(1.00, 0.60, 0.00),
        GameFaction::Assyria => Color::srgb(0.20, 0.40, 1.00),
        GameFaction::Babylon => Color::srgb(0.80, 1.00, 1.00),
        GameFaction::Egypt => Color::srgb(1.00, 1.00, 0.80),
    }
}

/// Background colour for a track cell, by its epoch band (START is distinct).
fn cell_color(index: u32, epoch: AstEpoch) -> Color {
    if index == 0 {
        return Color::srgb(0.45, 0.45, 0.45);
    }
    match epoch {
        AstEpoch::StoneAge => Color::srgb(0.55, 0.45, 0.35),
        AstEpoch::EarlyBronze => Color::srgb(0.66, 0.46, 0.26),
        AstEpoch::LateBronze => Color::srgb(0.50, 0.33, 0.18),
        AstEpoch::EarlyIron => Color::srgb(0.45, 0.50, 0.56),
        AstEpoch::LateIron => Color::srgb(0.30, 0.34, 0.40),
    }
}

/// The age bands as `(label, first_index, count)`, mirroring the sheet header.
fn bands() -> [(&'static str, u32, u32); 6] {
    [
        ("START", 0, 1),
        ("STONE AGE", 1, 4),
        ("EARLY BRONZE\n(2 cities)", 5, 3),
        ("LATE BRONZE\n(3 cities; 3 groups)", 8, 3),
        ("EARLY IRON\n(4 cities; 9/5)", 11, 3),
        ("LATE IRON\n(5 cities; points)", 14, 3),
    ]
}

/// Builds the A.S.T. panel once when the game starts.
pub fn spawn_ast_ui(
    commands: Commands,
    track: Res<AstTrack>,
    player_query: Query<(Entity, &Faction), With<Player>>,
    ui_theme: Res<LavaTheme>,
) {
    let mut ui = UIBuilder::new(commands, Some(ui_theme.clone()));

    ui.insert(AstUiRoot)
        .insert(Name::new("ast_ui_root"))
        .insert(ZIndex(Z_PANEL))
        .insert(Pickable::IGNORE)
        .set_node(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        })
        .flex_column()
        .padding_all_px(6.0)
        .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.9))
        .border_radius_all_px(5.0);

    // Title.
    ui.with_child(|t| {
        t.with_text("A.S.T.", Some(TextStyle::size_color(14.0, Color::WHITE)));
    });

    // Age-band header — each band spans `count` cells.
    ui.with_child(|header| {
        header.flex_row();
        for (label, _first, count) in bands() {
            header.with_child(|band| {
                band.width_px(CELL_W * count as f32)
                    .height_px(26.0)
                    .justify_center()
                    .align_items_center()
                    .with_text(label, Some(TextStyle::size_color(8.0, Color::WHITE)));
            });
        }
    });

    // Track row — 17 cells, plus absolutely-positioned markers.
    let spaces: Vec<_> = track.spaces.clone();
    ui.with_child(|row| {
        row.insert(AstTrackRow)
            .insert(Name::new("ast_track_row"))
            .flex_row()
            .modify_node(|mut n| n.position_type = PositionType::Relative);

        for space in &spaces {
            let label = if space.is_finish {
                "🏁".to_string()
            } else {
                space.index.to_string()
            };
            row.with_child(|cell| {
                cell.insert(AstCell { index: space.index })
                    .width_px(CELL_W - 2.0)
                    .height_px(CELL_H)
                    .margin_all_px(1.0)
                    .justify_center()
                    .align_items_center()
                    .bg_color(cell_color(space.index, space.epoch))
                    .border_radius_all_px(3.0)
                    .with_text(label, Some(TextStyle::size_color(11.0, Color::WHITE)));
            });
        }

        // Marker chips (one per player), positioned by the update system.
        for (player, faction) in player_query.iter() {
            row.with_child(|chip| {
                chip.insert(AstMarker { player })
                    .insert(Pickable::IGNORE)
                    .modify_node(|mut n| n.position_type = PositionType::Absolute)
                    .left(Val::Px(0.0))
                    .top(Val::Px(0.0))
                    .width_px(MARKER)
                    .height_px(MARKER)
                    .bg_color(ast_faction_color(faction.faction))
                    .border_radius_all_px(MARKER / 2.0);
            });
        }
    });

    ui.build();
}

/// Toggles the A.S.T. panel's visibility with F8.
pub fn toggle_ast_ui(
    keys: Res<ButtonInput<KeyCode>>,
    mut root_query: Query<&mut Visibility, With<AstUiRoot>>,
) {
    if !keys.just_pressed(KeyCode::F8) {
        return;
    }
    for mut visibility in &mut root_query {
        *visibility = match *visibility {
            Visibility::Hidden => Visibility::Inherited,
            _ => Visibility::Hidden,
        };
    }
}

/// Repositions each faction marker over its player's current space, stacking
/// markers that share a space.
pub fn update_ast_markers(
    player_query: Query<(Entity, &AstPosition), With<Player>>,
    mut marker_query: Query<(&AstMarker, &mut Node)>,
) {
    // Snapshot positions, then assign a deterministic stack order per space.
    let mut positions: Vec<(Entity, u32)> =
        player_query.iter().map(|(e, pos)| (e, pos.space)).collect();
    positions.sort_by_key(|(e, space)| (*space, e.index()));

    for (marker, mut node) in &mut marker_query {
        let Some(space) = positions
            .iter()
            .find(|(e, _)| *e == marker.player)
            .map(|(_, s)| *s)
        else {
            continue;
        };
        let stack = positions
            .iter()
            .filter(|(_, s)| *s == space)
            .position(|(e, _)| *e == marker.player)
            .unwrap_or(0);
        let col = (stack % STACK_COLS) as f32;
        let row = (stack / STACK_COLS) as f32;

        let grid_width = STACK_COLS as f32 * STACK_STEP - MARKER_GAP;
        let base_left = space as f32 * CELL_W + (CELL_W - grid_width) / 2.0 - 1.0;
        node.left = Val::Px(base_left + col * STACK_STEP);
        node.top = Val::Px(2.0 + row * STACK_STEP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::concepts::succession::succession_components::AST_LEN;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::App;

    fn ui_app() -> App {
        let mut app = App::new();
        app.insert_resource(LavaTheme::default());
        app.insert_resource(AstTrack::standard());
        app
    }

    fn spawn_player(app: &mut App, faction: GameFaction, space: u32) -> Entity {
        app.world_mut()
            .spawn((Player, Faction::new(faction), AstPosition::new(space)))
            .id()
    }

    #[test]
    fn builds_one_cell_per_space_and_a_marker_per_player() {
        let mut app = ui_app();
        spawn_player(&mut app, GameFaction::Egypt, 0);
        spawn_player(&mut app, GameFaction::Crete, 3);

        app.world_mut().run_system_once(spawn_ast_ui).unwrap();

        let cells = app
            .world_mut()
            .query::<&AstCell>()
            .iter(app.world())
            .count();
        let markers = app
            .world_mut()
            .query::<&AstMarker>()
            .iter(app.world())
            .count();
        let roots = app
            .world_mut()
            .query::<&AstUiRoot>()
            .iter(app.world())
            .count();

        assert_eq!(cells, AST_LEN, "one cell per AST space");
        assert_eq!(markers, 2, "one marker per player");
        assert_eq!(roots, 1, "single panel root");
    }

    #[test]
    fn markers_sharing_a_space_do_not_overlap() {
        let mut app = ui_app();
        // Two factions on the same space -> arranged into distinct grid cells
        // (first row, side by side) rather than one directly on top of the other.
        let a = spawn_player(&mut app, GameFaction::Egypt, 5);
        let b = spawn_player(&mut app, GameFaction::Crete, 5);

        app.world_mut().run_system_once(spawn_ast_ui).unwrap();
        app.world_mut().run_system_once(update_ast_markers).unwrap();

        let mut q = app.world_mut().query::<(&AstMarker, &Node)>();
        let mut by_player = std::collections::HashMap::new();
        for (m, n) in q.iter(app.world()) {
            by_player.insert(m.player, (n.left, n.top));
        }
        let (left_a, top_a) = by_player[&a];
        let (left_b, top_b) = by_player[&b];
        assert_eq!(top_a, top_b, "first two stacked markers share a row");
        assert_ne!(
            left_a, left_b,
            "first two stacked markers sit in different columns"
        );
    }

    #[test]
    fn a_large_stack_never_places_two_markers_in_the_same_spot() {
        let mut app = ui_app();
        // All 9 factions starting on the same space -- the worst-case stack.
        let players: Vec<Entity> = [
            GameFaction::Egypt,
            GameFaction::Crete,
            GameFaction::Africa,
            GameFaction::Asia,
            GameFaction::Assyria,
            GameFaction::Babylon,
            GameFaction::Illyria,
            GameFaction::Iberia,
            GameFaction::Thrace,
        ]
        .into_iter()
        .map(|f| spawn_player(&mut app, f, 0))
        .collect();
        assert_eq!(players.len(), 9);

        app.world_mut().run_system_once(spawn_ast_ui).unwrap();
        app.world_mut().run_system_once(update_ast_markers).unwrap();

        let mut q = app.world_mut().query::<(&AstMarker, &Node)>();
        let positions: Vec<(Val, Val)> =
            q.iter(app.world()).map(|(_, n)| (n.left, n.top)).collect();
        assert_eq!(positions.len(), 9);

        let mut seen = std::collections::HashSet::new();
        for pos in &positions {
            // `Val::Px` isn't `Hash`/`Eq`, so compare via its `f32` payload.
            let key = match pos {
                (Val::Px(l), Val::Px(t)) => (l.to_bits(), t.to_bits()),
                _ => panic!("expected Val::Px for both axes"),
            };
            assert!(
                seen.insert(key),
                "two markers landed on the exact same spot: {pos:?}"
            );
        }
    }
}
