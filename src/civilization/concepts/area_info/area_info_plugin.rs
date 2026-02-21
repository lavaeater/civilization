use bevy::prelude::*;
use lava_ui_builder::{UIBuilder, LavaTheme};

use crate::civilization::components::{
    BuiltCity, Faction, GameArea, GameCamera, Population,
};
use crate::player::Player;
use crate::{GameActivity, GameState};

pub struct AreaInfoPlugin;

#[derive(Component)]
pub struct AreaInfoRoot;

#[derive(Component)]
pub struct AreaInfoMarker {
    pub area_entity: Entity,
}

impl Plugin for AreaInfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameActivity::StartGame), spawn_area_info_markers)
            .add_systems(
                Update,
                update_area_info_markers.run_if(in_state(GameState::Playing)),
            );
    }
}

fn area_info_name(area_id: i32) -> String {
    format!("area_info_{}", area_id)
}

fn area_info_text_name(area_id: i32) -> String {
    format!("area_info_text_{}", area_id)
}

fn spawn_area_info_markers(
    commands: Commands,
    area_query: Query<(Entity, &GameArea, &Population, Has<BuiltCity>)>,
    player_query: Query<(Entity, &Faction, &Name), With<Player>>,
    ui_theme: Res<LavaTheme>,
) {
    let mut ui = UIBuilder::new(commands, Some(ui_theme.clone()));

    ui.insert(AreaInfoRoot)
        .insert(Name::new("area_info_root"))
        .set_node(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        });

    for (area_entity, game_area, population, has_city) in area_query.iter() {
        let max_pop = population.max_population;
        let text_content =
            build_area_info_text(game_area.id, population, &player_query, has_city, max_pop);

        ui.with_child(|marker| {
            marker
                .insert(AreaInfoMarker { area_entity })
                .insert(Name::new(area_info_name(game_area.id)))
                .modify_node(|mut n| n.position_type = PositionType::Absolute)
                .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.7))
                .padding_all_px(2.0)
                .border_radius_all_px(3.0);

            marker.with_child(|text_node| {
                text_node
                    .insert(Name::new(area_info_text_name(game_area.id)))
                    .with_text(text_content, None, Some(10.0), Some(Color::WHITE), None, None);
            });
        });
    }

    ui.build();
}

fn update_area_info_markers(
    area_query: Query<(Entity, &GameArea, &Population, &GlobalTransform, Has<BuiltCity>)>,
    player_query: Query<(Entity, &Faction, &Name), With<Player>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut marker_query: Query<(&AreaInfoMarker, &mut Node)>,
    mut text_query: Query<(&Name, &mut Text)>,
    ui_scale: Res<UiScale>,
) {
    let Ok((camera, camera_gt)) = camera_query.single() else {
        return;
    };

    let scale = ui_scale.0;

    for (area_entity, game_area, population, area_gt, has_city) in area_query.iter() {
        let max_pop = population.max_population;
        let text_content =
            build_area_info_text(game_area.id, population, &player_query, has_city, max_pop);

        let expected_text_name = area_info_text_name(game_area.id);
        for (name, mut text) in text_query.iter_mut() {
            if name.as_str() == expected_text_name {
                text.0 = text_content.clone();
            }
        }

        let world_pos = area_gt.translation();
        if let Ok(viewport_pos) = camera.world_to_viewport(camera_gt, world_pos) {
            for (marker, mut node) in marker_query.iter_mut() {
                if marker.area_entity == area_entity {
                    node.left = Val::Px(viewport_pos.x / scale - 20.0);
                    node.top = Val::Px(viewport_pos.y / scale + 30.0);
                }
            }
        }
    }
}

fn build_area_info_text(
    area_id: i32,
    population: &Population,
    player_query: &Query<(Entity, &Faction, &Name), With<Player>>,
    has_city: bool,
    max_pop: usize,
) -> String {
    let mut lines = Vec::new();

    let total_pop = population.total_population();
    let city_marker = if has_city { " 🏙" } else { "" };

    lines.push(format!("{}{} [{}/{}]", area_id, city_marker, total_pop, max_pop));

    for (player_entity, faction, _player_name) in player_query.iter() {
        let count = population.population_for_player(player_entity);
        if count > 0 {
            lines.push(format!("  {:?}: {}", faction.faction, count));
        }
    }

    lines.join("\n")
}
