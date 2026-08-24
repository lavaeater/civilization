use crate::civilization::components::GameCamera;
use crate::civilization::concepts::map::{Area, CameraFocusQueue, Map};
use crate::civilization::concepts::map_editor::map_editor_components::*;
use crate::loading::TextureAssets;
use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const CLICK_RADIUS: f32 = 30.0;
const REMOVE_CLICK_RADIUS: f32 = 12.0;
pub const MAP_EDITOR_ASSET_PATH: &str = "maps/civilization.map.ron";
const MAP_EDITOR_FILE_PATH: &str = "assets/maps/civilization.map.ron";

#[derive(Resource)]
pub struct MapEditorMapHandle(pub Handle<Map>);

pub fn load_map_editor_handle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MapEditorMapHandle(
        asset_server.load(MAP_EDITOR_ASSET_PATH),
    ));
}

/// Polls until the map asset is loaded, then spawns one entity per area
/// (background sprite + a clickable dot marker each) and centers the camera.
/// Runs every frame in `GameState::MapEditor` and no-ops once areas exist.
pub fn spawn_map_editor_areas(
    mut commands: Commands,
    handle: Option<Res<MapEditorMapHandle>>,
    maps: Res<Assets<Map>>,
    textures: Res<TextureAssets>,
    images: Res<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&mut Projection, &mut Transform), With<GameCamera>>,
    existing_areas: Query<(), With<EditableArea>>,
    mut state: ResMut<MapEditorState>,
) {
    if !existing_areas.is_empty() {
        return;
    }
    let Some(handle) = handle else { return };
    let Some(level) = maps.get(handle.0.id()) else {
        return;
    };

    commands.spawn((
        MapEditorRoot,
        Sprite {
            image: textures.map.clone(),
            ..default()
        },
        Transform::from_xyz(1250.0, 662.5, -1.0),
    ));

    if let (Some(img), Ok(window)) = (images.get(&textures.map), windows.single())
        && let Ok((mut projection, mut transform)) = camera.single_mut()
    {
        let img_size = img.texture_descriptor.size;
        let map_size = Vec2::new(img_size.width as f32, img_size.height as f32);
        let window_size = Vec2::new(window.resolution.width(), window.resolution.height());
        transform.translation = Vec3::new(1250.0, 662.5, transform.translation.z);
        let padding = 1.02;
        let needed_scale = (map_size.x / window_size.x.max(1.0))
            .max(map_size.y / window_size.y.max(1.0))
            * padding;
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = needed_scale;
        }
    }

    let mut first: Option<Entity> = None;
    for area in &level.areas {
        let entity = commands
            .spawn((
                Name::new(format!("MapEditorArea:{}", area.id)),
                Sprite {
                    image: textures.dot.clone(),
                    ..default()
                },
                Transform::from_xyz(area.x, area.y, 1.0),
                EditableArea {
                    id: area.id,
                    max_population: area.max_population,
                    land_connections: area.land_connections.clone(),
                    sea_connections: area.sea_connections.clone(),
                    start_area: area.start_area,
                    city_site: area.city_site,
                    flood_plain: area.flood_plain,
                    city_flood: area.city_flood,
                    volcano: area.volcano,
                },
            ))
            .id();
        if first.is_none() || area.id == 1 {
            first = Some(entity);
        }
    }
    state.current_area = first;
    state.dirty = false;
    state.status = None;
}

pub fn draw_map_editor_gizmos(
    mut gizmos: Gizmos,
    state: Res<MapEditorState>,
    areas: Query<(Entity, &EditableArea, &Transform)>,
) {
    let by_id: bevy::platform::collections::HashMap<i32, Vec2> = areas
        .iter()
        .map(|(_, a, t)| (a.id, t.translation.truncate()))
        .collect();

    let land_color = Color::srgb(0.85, 0.55, 0.15);
    let sea_color = Color::srgb(0.2, 0.55, 0.95);
    let dim_alpha = 0.25;

    let current = state
        .current_area
        .and_then(|e| areas.get(e).ok())
        .map(|(_, a, t)| (a, t.translation.truncate()));

    for (_, area, transform) in &areas {
        let from = transform.translation.truncate();
        let is_current = current.is_some_and(|(a, _)| a.id == area.id);
        for &target_id in &area.land_connections {
            let Some(&to) = by_id.get(&target_id) else {
                continue;
            };
            let color = if is_current {
                land_color
            } else {
                land_color.with_alpha(dim_alpha)
            };
            draw_fat_arrow(&mut gizmos, from, to, color, connection_lane_offset(ConnectionKind::Land));
        }
        for &target_id in &area.sea_connections {
            let Some(&to) = by_id.get(&target_id) else {
                continue;
            };
            let color = if is_current {
                sea_color
            } else {
                sea_color.with_alpha(dim_alpha)
            };
            draw_fat_arrow(&mut gizmos, from, to, color, connection_lane_offset(ConnectionKind::Sea));
        }
    }

    // Highlight ring + per-connection remove markers for the current area only.
    if let Some((area, pos)) = current {
        gizmos.circle_2d(pos, 18.0, Color::srgb(1.0, 1.0, 0.3));
        for &target_id in &area.land_connections {
            if let Some(&to) = by_id.get(&target_id) {
                draw_remove_marker(&mut gizmos, connection_midpoint(pos, to, ConnectionKind::Land));
            }
        }
        for &target_id in &area.sea_connections {
            if let Some(&to) = by_id.get(&target_id) {
                draw_remove_marker(&mut gizmos, connection_midpoint(pos, to, ConnectionKind::Sea));
            }
        }
    }

    if let Some(kind) = state.pending_connection {
        let color = match kind {
            ConnectionKind::Land => land_color,
            ConnectionKind::Sea => sea_color,
        };
        if let Some((_, pos)) = current {
            gizmos.circle_2d(pos, 26.0, color);
        }
    }
}

/// Land and sea arrows between the same two areas would otherwise be drawn on
/// the exact same line, so whichever type draws last (sea) hides the other.
/// Give each connection type its own parallel lane, offset perpendicular to
/// the line, so both stay visible.
fn connection_lane_offset(kind: ConnectionKind) -> f32 {
    match kind {
        ConnectionKind::Land => -7.0,
        ConnectionKind::Sea => 7.0,
    }
}

fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
    (a + b) * 0.5
}

/// Midpoint of a connection's arrow, in its own lane -- matches where
/// `draw_fat_arrow` actually draws it, so the remove-marker gizmo (and its
/// click target) lines up with what's on screen.
fn connection_midpoint(a: Vec2, b: Vec2, kind: ConnectionKind) -> Vec2 {
    let dir = (b - a).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x);
    midpoint(a, b) + perp * connection_lane_offset(kind)
}

/// Draws an arrow as three parallel offset lines so it reads clearly at map
/// scale, plus a triangular head at the target end. `lane_offset` shifts the
/// whole arrow perpendicular to its direction, so land/sea connections
/// between the same two areas run side by side instead of overlapping.
fn draw_fat_arrow(gizmos: &mut Gizmos, from: Vec2, to: Vec2, color: Color, lane_offset: f32) {
    let dir = (to - from).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x);
    let lane = perp * lane_offset;
    let from = from + lane;
    let to = to + lane;
    for offset in [-2.5, 0.0, 2.5] {
        gizmos.line_2d(from + perp * offset, to + perp * offset, color);
    }
    let head_len = 18.0;
    let head_width = 10.0;
    let back = to - dir * head_len;
    gizmos.line_2d(to, back + perp * head_width, color);
    gizmos.line_2d(to, back - perp * head_width, color);
}

fn draw_remove_marker(gizmos: &mut Gizmos, at: Vec2) {
    let s = REMOVE_CLICK_RADIUS * 0.7;
    let color = Color::srgb(1.0, 0.2, 0.2);
    gizmos.line_2d(at + Vec2::new(-s, -s), at + Vec2::new(s, s), color);
    gizmos.line_2d(at + Vec2::new(-s, s), at + Vec2::new(s, -s), color);
    gizmos.circle_2d(at, REMOVE_CLICK_RADIUS, color);
}

/// Left click handling, in priority order: remove a connection whose "x"
/// marker was clicked, else complete a pending add-connection onto the
/// clicked area, else just select the clicked area for navigation.
pub fn handle_map_editor_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut areas: Query<(Entity, &mut EditableArea, &Transform)>,
    mut state: ResMut<MapEditorState>,
    mut focus: ResMut<CameraFocusQueue>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
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

    // 1) Removing a connection from the current area.
    if let Some(current_entity) = state.current_area {
        let current_data = areas
            .get(current_entity)
            .ok()
            .map(|(_, a, t)| (a.clone(), t.translation.truncate()));
        if let Some((current_area, current_pos)) = current_data {
            let by_id: bevy::platform::collections::HashMap<i32, Vec2> = areas
                .iter()
                .map(|(_, a, t)| (a.id, t.translation.truncate()))
                .collect();

            for (kind, list) in [
                (ConnectionKind::Land, &current_area.land_connections),
                (ConnectionKind::Sea, &current_area.sea_connections),
            ] {
                for &target_id in list {
                    let Some(&to) = by_id.get(&target_id) else {
                        continue;
                    };
                    if world_pos.distance(connection_midpoint(current_pos, to, kind)) <= REMOVE_CLICK_RADIUS {
                        if let Ok((_, mut area, _)) = areas.get_mut(current_entity) {
                            match kind {
                                ConnectionKind::Land => area.land_connections.retain(|&id| id != target_id),
                                ConnectionKind::Sea => area.sea_connections.retain(|&id| id != target_id),
                            }
                            state.dirty = true;
                            state.status = Some(format!("Removed connection to area {target_id}"));
                        }
                        return;
                    }
                }
            }
        }
    }

    // 2) Find the area under the cursor, if any.
    let clicked = areas
        .iter()
        .map(|(e, a, t)| (e, a.id, world_pos.distance(t.translation.truncate())))
        .filter(|&(_, _, d)| d <= CLICK_RADIUS)
        .min_by(|a, b| a.2.total_cmp(&b.2));

    let Some((clicked_entity, clicked_id, _)) = clicked else {
        return;
    };

    if let Some(kind) = state.pending_connection {
        let Some(current_entity) = state.current_area else {
            return;
        };
        if clicked_entity == current_entity {
            return;
        }
        let Ok((_, current_area, _)) = areas.get(current_entity) else {
            return;
        };
        let already_connected = match kind {
            ConnectionKind::Land => current_area.land_connections.contains(&clicked_id),
            ConnectionKind::Sea => current_area.sea_connections.contains(&clicked_id),
        };
        if let Ok((_, mut current_area, _)) = areas.get_mut(current_entity)
            && !already_connected
        {
            match kind {
                ConnectionKind::Land => current_area.land_connections.push(clicked_id),
                ConnectionKind::Sea => current_area.sea_connections.push(clicked_id),
            }
        }
        let current_id = areas
            .get(current_entity)
            .map(|(_, a, _)| a.id)
            .unwrap_or_default();
        // Connections are stored symmetrically in the map file -- mirror it
        // on the target area too so both areas agree they're connected.
        if let Ok((_, mut target_area, _)) = areas.get_mut(clicked_entity) {
            let already = match kind {
                ConnectionKind::Land => target_area.land_connections.contains(&current_id),
                ConnectionKind::Sea => target_area.sea_connections.contains(&current_id),
            };
            if !already {
                match kind {
                    ConnectionKind::Land => target_area.land_connections.push(current_id),
                    ConnectionKind::Sea => target_area.sea_connections.push(current_id),
                }
            }
        }
        state.pending_connection = None;
        state.dirty = true;
        state.status = Some(format!(
            "Added {kind:?} connection: {current_id} <-> {clicked_id}"
        ));
        return;
    }

    state.current_area = Some(clicked_entity);
    if let Ok((_, _, transform)) = areas.get(clicked_entity) {
        focus.add_focus(transform.translation, 0.0, "Map editor: selected area");
    }
}

/// Writes the current editor state back to `Assets<Map>` (so re-entering the
/// editor or starting a new game this session sees it) and to the RON file
/// on disk (so it persists across restarts).
pub fn save_map_editor(
    areas: Query<(&EditableArea, &Transform)>,
    handle: Option<Res<MapEditorMapHandle>>,
    mut maps: ResMut<Assets<Map>>,
    mut state: ResMut<MapEditorState>,
) {
    if !state.save_requested {
        return;
    }
    state.save_requested = false;

    let mut area_list: Vec<Area> = areas
        .iter()
        .map(|(a, t)| Area {
            id: a.id,
            x: t.translation.x,
            y: t.translation.y,
            max_population: a.max_population,
            land_connections: a.land_connections.clone(),
            sea_connections: a.sea_connections.clone(),
            start_area: a.start_area,
            city_site: a.city_site,
            flood_plain: a.flood_plain,
            city_flood: a.city_flood,
            volcano: a.volcano,
        })
        .collect();
    area_list.sort_by_key(|a| a.id);

    let map = Map { areas: area_list };

    let Ok(ron_string) = ron::ser::to_string_pretty(&map, ron::ser::PrettyConfig::default())
    else {
        state.status = Some("Save failed: could not serialize map".to_string());
        return;
    };

    if let Err(err) = std::fs::write(MAP_EDITOR_FILE_PATH, ron_string) {
        state.status = Some(format!("Save failed: {err}"));
        return;
    }

    if let Some(handle) = handle
        && let Some(mut asset) = maps.get_mut(handle.0.id())
    {
        *asset = map;
    }

    state.dirty = false;
    state.status = Some("Saved.".to_string());
}

pub fn cleanup_map_editor(
    mut commands: Commands,
    roots: Query<Entity, With<MapEditorRoot>>,
    areas: Query<Entity, With<EditableArea>>,
    mut state: ResMut<MapEditorState>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    for entity in &areas {
        commands.entity(entity).despawn();
    }
    *state = MapEditorState::default();
}

/// Z/X to zoom, arrow keys to pan -- same convention as the in-game map
/// camera controls (`handle_map_camera_controls`).
pub fn handle_map_editor_camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<(&mut Projection, &mut Transform), With<GameCamera>>,
) {
    let Ok((mut projection, mut transform)) = camera.single_mut() else {
        return;
    };
    let Projection::Orthographic(ref mut ortho) = *projection else {
        return;
    };

    let dt = time.delta_secs();

    let zoom_speed = 1.5;
    let min_scale = 0.2;
    let max_scale = 3.0;
    if keyboard.pressed(KeyCode::KeyZ) {
        ortho.scale = (ortho.scale / (1.0 + zoom_speed * dt)).max(min_scale);
    }
    if keyboard.pressed(KeyCode::KeyX) {
        ortho.scale = (ortho.scale * (1.0 + zoom_speed * dt)).min(max_scale);
    }

    let pan_speed = 500.0 * ortho.scale;
    if keyboard.pressed(KeyCode::ArrowUp) {
        transform.translation.y += pan_speed * dt;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= pan_speed * dt;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= pan_speed * dt;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        transform.translation.x += pan_speed * dt;
    }
}

/// Nudges the camera by a fixed step, scaled by current zoom so panning feels
/// consistent whether zoomed in or out. Used by the on-screen arrow buttons.
pub fn pan_map_editor_camera(direction: Vec2, camera: &mut Query<(&Projection, &mut Transform), With<GameCamera>>) {
    let Ok((projection, mut transform)) = camera.single_mut() else {
        return;
    };
    let scale = if let Projection::Orthographic(ortho) = projection {
        ortho.scale
    } else {
        1.0
    };
    transform.translation += (direction * 120.0 * scale).extend(0.0);
}
