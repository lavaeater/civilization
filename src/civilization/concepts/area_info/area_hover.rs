//! Point at an area to see what is on it: whose city, everyone's tokens,
//! ships, and its terrain. Lines are tinted in the owners' colours.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::civilization::TokenMoveAnimation;
use crate::civilization::Z_DIALOG;
use crate::civilization::components::{
    BarbarianToken, BuiltCity, CityFlood, CitySite, CityToken, Faction, FloodPlain, GameArea,
    GameCamera, Population, Token, TokenStock, Treasury, Volcano,
};
use crate::civilization::concepts::ships::{PlayerShips, Ship};
use crate::civilization::concepts::succession::ast_ui_systems::ast_faction_color;
use bevy::platform::collections::HashMap;

/// How close (world units) the cursor must be to an area's centre.
const HOVER_RADIUS: f32 = 34.0;
const PLAIN: Color = Color::srgb(0.85, 0.85, 0.85);
const DIM: Color = Color::srgb(0.6, 0.6, 0.6);
const STRAY: Color = Color::srgb(1.0, 0.35, 0.35);

/// A drawn piece on the board, as the stray audit sees it.
pub type DrawnPiece = (
    Entity,
    &'static Transform,
    Option<&'static Visibility>,
    Option<&'static Name>,
    Option<&'static Token>,
    Option<&'static CityToken>,
    Has<BarbarianToken>,
    Option<&'static Ship>,
    Has<TokenMoveAnimation>,
);

type AreaRow<'a> = (
    Entity,
    &'a GameArea,
    &'a Name,
    &'a Population,
    &'a Transform,
    Option<&'a BuiltCity>,
    bool,
    bool,
    bool,
    bool,
);

/// Every piece drawn on `area` (closest to its centre) that the rules don't
/// put there, with where the rules do have it. Empty when all is well.
fn audit_drawn_pieces(
    area: Entity,
    areas: &Query<(
        Entity,
        &GameArea,
        &Name,
        &Population,
        &Transform,
        Option<&BuiltCity>,
        Has<CitySite>,
        Has<Volcano>,
        Has<FloodPlain>,
        Has<CityFlood>,
    )>,
    drawn: &Query<DrawnPiece, (With<Sprite>, Without<GameArea>, Without<AreaHoverPanel>)>,
    stocks: &Query<(Entity, &TokenStock, Option<&Treasury>)>,
    fleets: &Query<(Entity, &PlayerShips)>,
    who: &dyn Fn(Entity) -> (String, Option<Color>),
) -> Vec<String> {
    let rows: Vec<AreaRow> = areas.iter().collect();
    let nearest = |at: Vec2| {
        rows.iter()
            .min_by(|a, b| {
                a.4.translation
                    .truncate()
                    .distance(at)
                    .total_cmp(&b.4.translation.truncate().distance(at))
            })
            .map(|r| r.0)
    };
    let area_name = |e: Entity| {
        rows.iter()
            .find(|r| r.0 == e)
            .map_or_else(|| format!("{e:?}"), |r| r.2.to_string())
    };
    let mut token_home: HashMap<Entity, Entity> = HashMap::new();
    let mut city_home: HashMap<Entity, Entity> = HashMap::new();
    let ship_home: HashMap<Entity, Entity> = fleets
        .iter()
        .flat_map(|(_, fleet)| {
            fleet
                .ships_by_area
                .iter()
                .flat_map(|(area, ships)| ships.iter().map(move |ship| (*ship, *area)))
        })
        .collect();
    for row in &rows {
        for token in row.3.player_tokens().values().flatten() {
            token_home.insert(*token, row.0);
        }
        if let Some(city) = row.5 {
            city_home.insert(city.city, row.0);
        }
    }
    let off_board = |piece: Entity| -> Option<String> {
        stocks.iter().find_map(|(player, stock, treasury)| {
            if stock.iter().any(|t| t == piece) {
                Some(format!("in {}'s stock", who(player).0))
            } else if treasury.is_some_and(|t| t.iter().any(|t| t == piece)) {
                Some(format!("in {}'s treasury", who(player).0))
            } else {
                None
            }
        })
    };
    let mut out = Vec::new();
    for (piece, transform, visibility, name, token, city, barbarian, ship, sliding) in drawn {
        let at = transform.translation;
        if at.z < 0.0 || visibility == Some(&Visibility::Hidden) || sliding {
            continue; // the map itself; hidden pieces; pieces on their way
        }
        if nearest(at.truncate()) != Some(area) {
            continue;
        }
        let line = if let Some(token) = token {
            match token_home.get(&piece) {
                Some(home) if *home == area => continue,
                Some(home) => format!(
                    "token {piece:?} of {}: the rules have it in {}",
                    who(token.player()).0,
                    area_name(*home)
                ),
                None => format!(
                    "token {piece:?} of {}: {}",
                    who(token.player()).0,
                    off_board(piece).unwrap_or_else(|| "on no area, in no stock".into())
                ),
            }
        } else if let Some(city) = city {
            match city_home.get(&piece) {
                Some(home) if *home == area => continue,
                Some(home) => format!(
                    "city {piece:?} of {}: belongs to {}",
                    who(city.player).0,
                    area_name(*home)
                ),
                None => format!(
                    "city {piece:?} of {}: no area has it as its city",
                    who(city.player).0
                ),
            }
        } else if barbarian {
            match token_home.get(&piece) {
                Some(home) if *home == area => continue,
                Some(home) => format!(
                    "barbarian {piece:?}: the rules have it in {}",
                    area_name(*home)
                ),
                None => format!("barbarian {piece:?}: on no area"),
            }
        } else if let Some(ship) = ship {
            match ship_home.get(&piece) {
                Some(home) if *home == area => continue,
                Some(home) => format!(
                    "ship {piece:?} of {}: the rules have it at {}",
                    who(ship.owner).0,
                    area_name(*home)
                ),
                None => format!(
                    "ship {piece:?} of {}: not on the board (in stock?)",
                    who(ship.owner).0
                ),
            }
        } else {
            format!(
                "sprite {piece:?} '{}' is neither token, city, barbarian nor ship",
                name.map_or("unnamed", |n| n.as_str())
            )
        };
        out.push(line);
    }
    out.sort();
    out
}

#[derive(Component)]
pub struct AreaHoverPanel;

/// One line of the panel and its colour.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverLine(pub String, pub Color);

pub fn spawn_area_hover_panel(mut commands: Commands, existing: Query<(), With<AreaHoverPanel>>) {
    if !existing.is_empty() {
        return;
    }
    commands.spawn((
        AreaHoverPanel,
        Name::new("area_hover_panel"),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.92)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.25)),
        GlobalZIndex(Z_DIALOG),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}

/// The lines describing one area.
#[allow(clippy::too_many_arguments)]
pub fn describe_area(
    area_name: &str,
    population: &Population,
    city_owner: Option<(&str, Option<Color>)>,
    city_site: bool,
    hazards: &[&str],
    tokens: &[(String, Option<Color>, usize)],
    ships: &[(String, Option<Color>, usize)],
) -> Vec<HoverLine> {
    let mut lines = vec![HoverLine(area_name.to_string(), Color::WHITE)];
    match city_owner {
        Some((owner, color)) => lines.push(HoverLine(
            format!("City of {owner}"),
            color.unwrap_or(PLAIN),
        )),
        None if city_site => lines.push(HoverLine("City site, empty".into(), DIM)),
        None => {}
    }
    lines.push(HoverLine(
        format!(
            "Population {}/{}",
            population.total_population(),
            population.max_population
        ),
        PLAIN,
    ));
    for (name, color, count) in tokens {
        lines.push(HoverLine(
            format!("  {name}: {count} token(s)"),
            color.unwrap_or(PLAIN),
        ));
    }
    for (name, color, count) in ships {
        lines.push(HoverLine(
            format!("  {name}: {count} ship(s)"),
            color.unwrap_or(PLAIN),
        ));
    }
    if !hazards.is_empty() {
        lines.push(HoverLine(hazards.join(", "), Color::srgb(0.95, 0.6, 0.4)));
    }
    lines
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_area_hover_panel(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    areas: Query<(
        Entity,
        &GameArea,
        &Name,
        &Population,
        &Transform,
        Option<&BuiltCity>,
        Has<CitySite>,
        Has<Volcano>,
        Has<FloodPlain>,
        Has<CityFlood>,
    )>,
    owners: Query<(&Name, Option<&Faction>)>,
    fleets: Query<(Entity, &PlayerShips)>,
    mut panel: Query<(Entity, &mut Node, &mut Visibility), With<AreaHoverPanel>>,
    ui_scale: Res<UiScale>,
    drawn: Query<DrawnPiece, (With<Sprite>, Without<GameArea>, Without<AreaHoverPanel>)>,
    stocks: Query<(Entity, &TokenStock, Option<&Treasury>)>,
    mut shown: Local<Vec<HoverLine>>,
) {
    let Ok((panel, mut node, mut visibility)) = panel.single_mut() else {
        return;
    };
    let hovered = (|| {
        let window = windows.single().ok()?;
        let cursor = window.cursor_position()?;
        let (camera, camera_transform) = camera.single().ok()?;
        let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
        let area = areas
            .iter()
            .map(|a| (a.4.translation.truncate().distance(world), a))
            .filter(|(d, _)| *d <= HOVER_RADIUS)
            .min_by(|a, b| a.0.total_cmp(&b.0))?
            .1;
        Some((cursor, area))
    })();
    let Some((
        cursor,
        (entity, game_area, name, population, _, city, city_site, volcano, flood_plain, city_flood),
    )) = hovered
    else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };

    let who = |player: Entity| -> (String, Option<Color>) {
        owners
            .get(player)
            .map_or(("?".to_string(), None), |(n, f)| {
                (n.to_string(), f.map(|f| ast_faction_color(f.faction)))
            })
    };
    let mut tokens: Vec<(String, Option<Color>, usize)> = population
        .player_tokens()
        .iter()
        .filter(|(_, t)| !t.is_empty())
        .map(|(player, t)| {
            let (n, c) = who(*player);
            (n, c, t.len())
        })
        .collect();
    tokens.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    let mut ships: Vec<(String, Option<Color>, usize)> = fleets
        .iter()
        .filter_map(|(player, fleet)| {
            let n = fleet.ships_in_area(entity).len();
            (n > 0).then(|| {
                let (name, color) = who(player);
                (name, color, n)
            })
        })
        .collect();
    ships.sort_by(|a, b| a.0.cmp(&b.0));
    let owner = city.map(|c| who(c.player));
    let mut hazards = Vec::new();
    if volcano {
        hazards.push("Volcano");
    }
    if city_flood {
        hazards.push("Flood plain (floods the city)");
    } else if flood_plain {
        hazards.push("Flood plain");
    }
    // Area names are "<id>:<place>"; show the place first.
    let title = match name.as_str().split_once(':') {
        Some((id, place)) => format!("{place} (area {id})"),
        None => format!("Area {}", game_area.id),
    };
    let mut lines = describe_area(
        &title,
        population,
        owner.as_ref().map(|(n, c)| (n.as_str(), *c)),
        city_site,
        &hazards,
        &tokens,
        &ships,
    );

    let strays = audit_drawn_pieces(entity, &areas, &drawn, &stocks, &fleets, &who);
    if !strays.is_empty() {
        lines.push(HoverLine(
            "Drawn here but not here by the rules:".into(),
            STRAY,
        ));
        lines.extend(strays.into_iter().map(|line| HoverLine(line, STRAY)));
    }

    let scale = ui_scale.0;
    node.left = Val::Px(cursor.x / scale + 18.0);
    node.top = Val::Px(cursor.y / scale + 18.0);
    visibility.set_if_neq(Visibility::Inherited);
    if *shown == lines {
        return;
    }
    shown.clone_from(&lines);
    for HoverLine(line, color) in lines.iter().skip(1) {
        if *color == STRAY {
            warn!("[STRAY] {title}: {}", line.trim());
        }
    }
    commands
        .entity(panel)
        .despawn_related::<Children>()
        .with_children(|panel| {
            for (i, HoverLine(text, color)) in lines.into_iter().enumerate() {
                panel.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: FontSize::Px(if i == 0 { 16.0 } else { 14.0 }),
                        ..default()
                    },
                    TextColor(color),
                    Pickable::IGNORE,
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_city_and_everyone_on_the_area_are_named() {
        let population = Population::new(3);
        let crimson = Color::srgb(1.0, 0.0, 0.0);
        let red = Some(crimson);
        let lines = describe_area(
            "Thebes (area 7)",
            &population,
            Some(("Egypt", red)),
            true,
            &["Flood plain"],
            &[("Crete".into(), None, 2)],
            &[("Egypt".into(), red, 1)],
        );
        let text: Vec<&str> = lines.iter().map(|l| l.0.as_str()).collect();
        assert_eq!(
            text,
            vec![
                "Thebes (area 7)",
                "City of Egypt",
                "Population 0/3",
                "  Crete: 2 token(s)",
                "  Egypt: 1 ship(s)",
                "Flood plain",
            ]
        );
        assert_eq!(
            lines[1].1, crimson,
            "the city line wears the owner's colour"
        );
    }
}
