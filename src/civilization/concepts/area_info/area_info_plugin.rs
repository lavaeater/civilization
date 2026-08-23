use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::ui::ZIndex;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

use crate::civilization::components::{
    BuiltCity, CityFlood, CitySite, Faction, FloodPlain, GameArea, GameCamera, Population, Volcano,
};
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::civilization::{MovementSelectionState, Z_AREA_INDICATOR};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
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
    format!("area_info_{area_id}")
}

fn area_info_text_name(area_id: i32) -> String {
    format!("area_info_text_{area_id}")
}

/// Static, map-defined terrain/site flags for an area, gathered up so the
/// text-building code doesn't need a growing pile of `Has<T>` parameters.
#[derive(Clone, Copy, Default)]
struct AreaTerrainFlags {
    has_city: bool,
    city_site: bool,
    volcano: bool,
    flood_plain: bool,
    city_flood: bool,
}

fn spawn_area_info_markers(
    commands: Commands,
    area_query: Query<(
        Entity,
        &GameArea,
        &Population,
        Has<BuiltCity>,
        Has<CitySite>,
        Has<Volcano>,
        Has<FloodPlain>,
        Has<CityFlood>,
    )>,
    player_query: Query<(Entity, &Faction, &Name), With<Player>>,
    ui_theme: Res<LavaTheme>,
) {
    let mut ui = UIBuilder::new(commands, Some(ui_theme.clone()));

    ui.insert(AreaInfoRoot)
        .insert(Name::new("area_info_root"))
        .insert(ZIndex(Z_AREA_INDICATOR))
        .insert(Pickable::IGNORE)
        .set_node(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        });

    for (area_entity, game_area, population, has_city, city_site, volcano, flood_plain, city_flood) in
        area_query.iter()
    {
        let max_pop = population.max_population;
        let flags = AreaTerrainFlags {
            has_city,
            city_site,
            volcano,
            flood_plain,
            city_flood,
        };
        let text_content = build_area_info_text(game_area.id, population, &player_query, flags, max_pop);

        ui.with_child(|marker| {
            marker
                .insert(AreaInfoMarker { area_entity })
                .insert(Name::new(area_info_name(game_area.id)))
                .modify_node(|mut n| n.position_type = PositionType::Absolute)
                .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.9))
                .padding_all_px(2.0)
                .border_radius_all_px(3.0);

            marker.with_child(|text_node| {
                text_node
                    .insert(Name::new(area_info_text_name(game_area.id)))
                    .with_text(
                        text_content,
                        Some(TextStyle::size_color(20.0, Color::WHITE)),
                    );
            });
        });
    }

    ui.build();
}

fn update_area_info_markers(
    area_query: Query<(
        Entity,
        &GameArea,
        &Population,
        &GlobalTransform,
        Has<BuiltCity>,
        Has<CitySite>,
        Has<Volcano>,
        Has<FloodPlain>,
        Has<CityFlood>,
    )>,
    player_query: Query<(Entity, &Faction, &Name), With<Player>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut marker_query: Query<(&AreaInfoMarker, &mut Node, &mut Visibility)>,
    mut text_query: Query<(&Name, &mut Text)>,
    ui_scale: Res<UiScale>,
    human_available_moves: Query<&AvailableMoves, With<IsHuman>>,
    movement_selection: Res<MovementSelectionState>,
) {
    let Ok((camera, camera_gt)) = camera_query.single() else {
        return;
    };

    let scale = ui_scale.0;
    let relevant_areas =
        relevant_areas_for_current_action(&human_available_moves, &movement_selection);

    // Visibility is independent of on-screen projection succeeding this
    // frame -- a marker whose area's viewport projection fails shouldn't
    // stay stuck showing (or hiding) whatever it last was.
    for (marker, _, mut visibility) in &mut marker_query {
        *visibility = if relevant_areas.contains(&marker.area_entity) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (area_entity, game_area, population, area_gt, has_city, city_site, volcano, flood_plain, city_flood) in
        area_query.iter()
    {
        let max_pop = population.max_population;
        let flags = AreaTerrainFlags {
            has_city,
            city_site,
            volcano,
            flood_plain,
            city_flood,
        };
        let text_content = build_area_info_text(game_area.id, population, &player_query, flags, max_pop);

        let expected_text_name = area_info_text_name(game_area.id);
        for (name, mut text) in &mut text_query {
            if name.as_str() == expected_text_name {
                text.0.clone_from(&text_content);
            }
        }

        let world_pos = area_gt.translation();
        if let Ok(viewport_pos) = camera.world_to_viewport(camera_gt, world_pos) {
            for (marker, mut node, _) in &mut marker_query {
                if marker.area_entity == area_entity {
                    node.left = Val::Px(viewport_pos.x / scale - 20.0);
                    node.top = Val::Px(viewport_pos.y / scale + 30.0);
                }
            }
        }
    }
}

/// Areas the human player could act on right now, going only by what's
/// actually in their `AvailableMoves` -- markers for everything else stay
/// hidden so the board reads as "what can I do" rather than "here is every
/// area's population, all the time." During Movement this narrows further
/// to the currently focused source (mirroring `draw_movement_arrows`, which
/// only draws arrows from that same source): showing every source a player
/// could still move from, all at once, would be exactly the clutter this is
/// meant to cut down on. Phases without a "which area" concept at all
/// (Trade, AcquireCivilizationCards) naturally end up with nothing relevant,
/// so their markers are all hidden -- there's no area to point at.
fn relevant_areas_for_current_action(
    human_available_moves: &Query<&AvailableMoves, With<IsHuman>>,
    movement_selection: &MovementSelectionState,
) -> HashSet<Entity> {
    let focused_source = movement_selection.current_source();
    let mut relevant = HashSet::default();
    for available_moves in human_available_moves {
        relevant_areas_from_moves(
            available_moves.moves.values(),
            focused_source,
            &mut relevant,
        );
    }
    relevant
}

/// Pure move-classification logic, split out from [`relevant_areas_for_current_action`]
/// so it's testable without a `Query`.
fn relevant_areas_from_moves<'a>(
    moves: impl Iterator<Item = &'a GameMove>,
    focused_source: Option<Entity>,
    relevant: &mut HashSet<Entity>,
) {
    for game_move in moves {
        match game_move {
            GameMove::PopulationExpansion(m) => {
                relevant.insert(m.area);
            }
            GameMove::CityConstruction(m) => {
                relevant.insert(m.target);
            }
            GameMove::EliminateCity(m) => {
                relevant.insert(m.area);
            }
            GameMove::Movement(m)
            | GameMove::ShipFerry(m)
            | GameMove::AttackArea(m)
            | GameMove::AttackCity(m) => {
                if let Some(focused) = focused_source
                    && m.source != focused
                {
                    continue;
                }
                relevant.insert(m.source);
                relevant.insert(m.target);
            }
            GameMove::EndMovement
            | GameMove::EndCityConstruction
            | GameMove::Trade(_)
            | GameMove::AcquireCivilizationCards(_) => {}
        }
    }
}

fn build_area_info_text(
    area_id: i32,
    population: &Population,
    player_query: &Query<(Entity, &Faction, &Name), With<Player>>,
    flags: AreaTerrainFlags,
    max_pop: usize,
) -> String {
    let mut lines = Vec::new();

    let total_pop = population.total_population();

    // City-site markers describe distinct things: a built city, an area
    // eligible to found one but not yet built, and terrain/calamity
    // hazards that can strike this area (volcano, flood plain, or a flood
    // plain that specifically threatens a city here). Plain-text tags
    // rather than emoji, since the game's fonts don't carry emoji glyphs.
    let mut markers = String::new();
    if flags.has_city {
        markers.push_str(" [City]");
    } else if flags.city_site {
        markers.push_str(" [Site]");
    }
    if flags.volcano {
        markers.push_str(" [Volcano]");
    }
    if flags.city_flood {
        markers.push_str(" [Flood/City]");
    } else if flags.flood_plain {
        markers.push_str(" [Flood]");
    }

    lines.push(format!("{area_id}{markers} [{total_pop}/{max_pop}]"));

    for (player_entity, faction, _player_name) in player_query.iter() {
        let count = population.population_for_player(player_entity);
        if count > 0 {
            lines.push(format!("  {:?}: {}", faction.faction, count));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::game_moves::{BuildCityMove, MovementMove, PopExpMove};

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn population_expansion_move_marks_its_area_relevant() {
        let moves = [GameMove::PopulationExpansion(PopExpMove::new(e(1), 3))];
        let mut relevant = HashSet::default();
        relevant_areas_from_moves(moves.iter(), None, &mut relevant);
        assert_eq!(relevant, HashSet::from_iter([e(1)]));
    }

    #[test]
    fn city_construction_move_marks_its_target_relevant() {
        let moves = [GameMove::CityConstruction(BuildCityMove::new(e(2), e(9)))];
        let mut relevant = HashSet::default();
        relevant_areas_from_moves(moves.iter(), None, &mut relevant);
        assert_eq!(relevant, HashSet::from_iter([e(2)]));
    }

    #[test]
    fn movement_moves_with_no_focused_source_mark_every_source_and_target() {
        let moves = [
            GameMove::Movement(MovementMove::new(e(1), e(2), e(9), 3)),
            GameMove::AttackArea(MovementMove::new(e(3), e(4), e(9), 2)),
        ];
        let mut relevant = HashSet::default();
        relevant_areas_from_moves(moves.iter(), None, &mut relevant);
        assert_eq!(relevant, HashSet::from_iter([e(1), e(2), e(3), e(4)]));
    }

    /// The core of this feature: with a focused source, only that source's
    /// own moves stay relevant -- other sources the player could still move
    /// from (but isn't currently looking at) are excluded, matching
    /// `draw_movement_arrows`'s behavior exactly.
    #[test]
    fn movement_moves_with_a_focused_source_only_mark_that_sources_targets() {
        let moves = [
            GameMove::Movement(MovementMove::new(e(1), e(2), e(9), 3)),
            GameMove::AttackCity(MovementMove::new(e(1), e(5), e(9), 1)),
            GameMove::ShipFerry(MovementMove::new(e(3), e(4), e(9), 2)),
        ];
        let mut relevant = HashSet::default();
        relevant_areas_from_moves(moves.iter(), Some(e(1)), &mut relevant);
        // Both of source e(1)'s moves are included (source + both targets)...
        assert_eq!(relevant, HashSet::from_iter([e(1), e(2), e(5)]));
        // ...but the unfocused source e(3) and its target e(4) are not.
        assert!(!relevant.contains(&e(3)));
        assert!(!relevant.contains(&e(4)));
    }

    #[test]
    fn end_turn_and_area_less_moves_mark_nothing_relevant() {
        let moves = [GameMove::EndMovement, GameMove::EndCityConstruction];
        let mut relevant = HashSet::default();
        relevant_areas_from_moves(moves.iter(), None, &mut relevant);
        assert!(relevant.is_empty());
    }
}
