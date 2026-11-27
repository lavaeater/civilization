use crate::GameActivity;
use crate::civilization::ShipStock;
use crate::civilization::components::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::ReturnCityToStock;
use crate::civilization::functions::return_token_to_stock;
use bevy::platform::collections::HashSet;
use bevy::prelude::{
    Add, Added, Changed, Commands, Entity, On, Or, Query, Res, Sprite, State, Transform,
    Visibility, With, warn,
};

pub fn on_add_return_token_to_stock(
    trigger: On<Add<ReturnTokenToStock>>,
    token_query: Query<&Token>,
    mut player_query: Query<(&mut TokenStock, &mut PlayerAreas)>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    commands
        .entity(entity)
        .remove::<(ReturnTokenToStock, Sprite, Transform, Visibility)>();

    if let Ok(token) = token_query.get(entity) {
        if let Ok((mut token_stock, mut player_areas)) = player_query.get_mut(token.player()) {
            let player = token.player();
            if player_areas.holds_token(entity) {
                record_loss(&mut commands, player, |l| &mut l.tokens);
            } else {
                record_loss(&mut commands, player, |l| &mut l.treasury);
            }
            return_token_to_stock(entity, &mut token_stock, &mut player_areas);
        } else {
            warn!(
                "[RETURN_TOKEN] Failed to get player components for token {:?}, player {:?}",
                entity,
                token.player()
            );
        }
    } else {
        warn!(
            "[RETURN_TOKEN] Failed to get Token component for entity {:?}",
            entity
        );
    }
}

/// Retires a city token: strips what makes it visible on the map and hands it
/// back to its owner's stock.
///
/// City removal used to update `CityTokenStock`/`PlayerCities` but leave the
/// token's `Sprite` and `Transform` in place, so a destroyed or reduced city
/// kept its icon sitting on the board -- the replacement population tokens
/// were placed underneath it and the whole calamity read, to the player, as
/// having done nothing at all.
pub fn on_add_return_city_to_stock(
    trigger: On<Add<ReturnCityToStock>>,
    city_token_query: Query<&CityToken>,
    mut owner_query: Query<(&mut CityTokenStock, &mut PlayerCities)>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    retire_city_token_visuals(&mut commands, entity);
    commands.entity(entity).remove::<ReturnCityToStock>();

    let Ok(city_token) = city_token_query.get(entity) else {
        warn!("[RETURN_CITY] {entity:?} is not a city token");
        return;
    };
    let Ok((mut city_stock, mut player_cities)) = owner_query.get_mut(city_token.player) else {
        warn!(
            "[RETURN_CITY] no stock/cities for {:?}, owner of city {entity:?}",
            city_token.player
        );
        return;
    };
    if player_cities.remove_city_by_token(entity).is_some() {
        record_loss(&mut commands, city_token.player, |l| &mut l.cities);
    }
    city_stock.return_token_to_stock(entity);
}

/// Removes the rendering components from a city token that has left the board.
/// Call this from any path that returns a city token to stock itself.
pub fn retire_city_token_visuals(commands: &mut Commands, city_token: Entity) {
    commands
        .entity(city_token)
        .remove::<(Sprite, Transform, Visibility)>();
}

type StockChanged = Or<(
    Changed<TokenStock>,
    Changed<Treasury>,
    Changed<CityTokenStock>,
    Changed<ShipStock>,
)>;

/// A piece in a stock or treasury is off the board, so it must not be drawn.
///
/// Several paths move tokens off the board by editing `TokenStock`/`Treasury`
/// directly instead of going through `ReturnTokenToStock` -- invaders bounced
/// off a city (`return_all_tokens_from_area_to_player`), Architecture's saved
/// token, pillage -- and those kept their `Sprite`, so a ghost population
/// marker stayed where the token last stood. Standing on a city's spot, it
/// made the city look like plain population. Rather than patch each path,
/// this enforces the invariant whenever a stock changes.
///
/// Ships too: a ship its owner could not maintain went back to `ShipStock`
/// but stayed drawn on the coast it last stood on.
pub fn hide_off_board_pieces(
    owners: Query<
        (
            Option<&TokenStock>,
            Option<&Treasury>,
            Option<&CityTokenStock>,
            Option<&ShipStock>,
        ),
        StockChanged,
    >,
    drawn: Query<(), With<Sprite>>,
    mut commands: Commands,
) {
    for (stock, treasury, city_stock, ship_stock) in &owners {
        let off_board = stock
            .into_iter()
            .flat_map(TokenStock::iter)
            .chain(treasury.into_iter().flat_map(Treasury::iter))
            .chain(city_stock.into_iter().flat_map(CityTokenStock::iter))
            .chain(ship_stock.into_iter().flat_map(|s| s.ships.iter().copied()));
        for piece in off_board.filter(|p| drawn.contains(*p)) {
            commands
                .entity(piece)
                .remove::<(Sprite, Transform, Visibility)>();
        }
    }
}

/// A drawn population token must stand in some area's `Population`.
///
/// `hide_off_board_pieces` catches tokens that went to a stock or treasury,
/// but a token can also drop out of every `Population` without reaching one
/// (the play-test showed stray markers on city sites). Anything drawn that is
/// on no area is a ghost: it is logged, so the path that left it can be
/// found, and hidden.
pub fn hide_tokens_off_the_board(
    changed: Query<(), Or<(Changed<Population>, Added<Sprite>)>>,
    areas: Query<&Population>,
    drawn: Query<(Entity, &Token), With<Sprite>>,
    activity: Option<Res<State<GameActivity>>>,
    mut commands: Commands,
) {
    if changed.is_empty() {
        return;
    }
    let on_board: HashSet<Entity> = areas
        .iter()
        .flat_map(|p| p.player_tokens().values().flatten().copied())
        .collect();
    for (entity, token) in &drawn {
        if !on_board.contains(&entity) {
            warn!(
                "[GHOST] token {entity:?} of {:?} is drawn but on no area (in {:?}) - hiding it",
                token.player(),
                activity.as_ref().map(|a| a.get().clone())
            );
            commands
                .entity(entity)
                .remove::<(Sprite, Transform, Visibility)>();
        }
    }
}

/// How far from its area's centre a token may be drawn (the layout puts
/// them within ~60 units).
const STRAY_DISTANCE: f32 = 90.0;

/// Diagnostic: a drawn piece must be drawn where the rules have it. Logs
/// `[STRAY]` for a token drawn far from the area whose `Population` holds it
/// (and not sliding there), and for a drawn city no area's `BuiltCity`
/// names. Each piece is reported once until it is right again.
#[allow(clippy::type_complexity)]
pub fn report_stray_pieces(
    areas: Query<(
        Entity,
        &GameArea,
        &Population,
        &Transform,
        Option<&BuiltCity>,
    )>,
    tokens: Query<
        (
            Entity,
            &Token,
            &Transform,
            bevy::prelude::Has<crate::civilization::TokenMoveAnimation>,
        ),
        (With<Sprite>, bevy::prelude::Without<GameArea>),
    >,
    cities: Query<
        (Entity, &CityToken, &Transform),
        (With<Sprite>, bevy::prelude::Without<GameArea>),
    >,
    activity: Option<Res<State<GameActivity>>>,
    mut reported: bevy::prelude::Local<HashSet<Entity>>,
    mut frame: bevy::prelude::Local<u32>,
) {
    // A diagnostic: every 15th frame is plenty.
    *frame = frame.wrapping_add(1);
    if *frame % 15 != 0 {
        return;
    }
    let mut home = bevy::platform::collections::HashMap::new();
    let mut city_sites = bevy::platform::collections::HashMap::new();
    for (area, game_area, pop, transform, city) in &areas {
        let at = transform.translation.truncate();
        for token in pop.player_tokens().values().flatten() {
            home.insert(*token, (area, game_area.id, at));
        }
        if let Some(city) = city {
            city_sites.insert(city.city, (game_area.id, at));
        }
    }
    let phase = activity.as_ref().map(|a| a.get().clone());
    let mut stray = HashSet::new();
    for (entity, token, transform, sliding) in &tokens {
        let Some((_, area_id, at)) = home.get(&entity) else {
            continue; // hide_tokens_off_the_board handles these
        };
        let drawn = transform.translation.truncate();
        if sliding {
            continue;
        }
        // Closer to another area's centre than to its own reads, on the
        // board, as standing in that other area.
        let drawn_near = areas
            .iter()
            .min_by(|a, b| {
                a.3.translation
                    .truncate()
                    .distance(drawn)
                    .total_cmp(&b.3.translation.truncate().distance(drawn))
            })
            .map(|a| a.1.id);
        if drawn.distance(*at) <= STRAY_DISTANCE && drawn_near == Some(*area_id) {
            continue;
        }
        stray.insert(entity);
        if reported.insert(entity) {
            warn!(
                "[STRAY] token {entity:?} of {:?} is in area {area_id} but drawn {:.0} away, near area {drawn_near:?} (in {phase:?})",
                token.player(),
                drawn.distance(*at),
            );
        }
    }
    for (entity, city, transform) in &cities {
        let drawn = transform.translation.truncate();
        let ok = city_sites
            .get(&entity)
            .is_some_and(|(_, at)| drawn.distance(*at) <= STRAY_DISTANCE);
        if ok {
            continue;
        }
        stray.insert(entity);
        if reported.insert(entity) {
            warn!(
                "[STRAY] city {entity:?} of {:?} is drawn at {drawn:?} but {} (in {phase:?})",
                city.player,
                city_sites.get(&entity).map_or_else(
                    || "no area has it as its BuiltCity".to_string(),
                    |(id, _)| format!("belongs to area {id}")
                ),
            );
        }
    }
    reported.retain(|e| stray.contains(e));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::World;

    fn drawn_token(world: &mut World) -> Entity {
        world.spawn((Sprite::default(), Transform::default())).id()
    }

    #[test]
    fn tokens_in_stock_or_treasury_and_cities_in_stock_are_not_drawn() {
        let mut world = World::new();
        let (in_stock, in_treasury, city) = (
            drawn_token(&mut world),
            drawn_token(&mut world),
            drawn_token(&mut world),
        );
        let on_board = drawn_token(&mut world);
        let mut treasury = Treasury::default();
        treasury.add_token_to_treasury(in_treasury);
        world.spawn((
            TokenStock::new(47, vec![in_stock]),
            treasury,
            CityTokenStock::new(9, vec![city]),
        ));

        world.run_system_once(hide_off_board_pieces).unwrap();

        for piece in [in_stock, in_treasury, city] {
            assert!(
                world.get::<Sprite>(piece).is_none(),
                "{piece:?} still drawn"
            );
        }
        assert!(world.get::<Sprite>(on_board).is_some());
    }

    /// A ship its owner could not maintain goes back to stock; it must
    /// leave the coast it stood on.
    #[test]
    fn an_unmaintained_ship_back_in_stock_is_not_drawn() {
        let mut world = World::new();
        let sunk = drawn_token(&mut world);
        let afloat = drawn_token(&mut world);
        world.spawn(ShipStock::new(vec![sunk]));

        world.run_system_once(hide_off_board_pieces).unwrap();

        assert!(
            world.get::<Sprite>(sunk).is_none(),
            "the returned ship is still drawn"
        );
        assert!(world.get::<Sprite>(afloat).is_some());
    }

    /// The repelled-invader path: tokens pushed straight back into stock.
    #[test]
    fn repelled_invaders_do_not_leave_ghosts_on_the_city() {
        let mut world = World::new();
        let player = world.spawn(TokenStock::new(47, vec![])).id();
        let area = world.spawn_empty().id();
        let invaders: Vec<Entity> = (0..3).map(|_| drawn_token(&mut world)).collect();
        let mut population = Population::new(3);
        let mut areas = PlayerAreas::default();
        for &t in &invaders {
            population.add_token_to_area(player, t);
            areas.add_token_to_area(area, t);
        }
        world.run_system_once(hide_off_board_pieces).unwrap();
        assert!(
            world.get::<Sprite>(invaders[0]).is_some(),
            "on board: drawn"
        );

        let mut stock = world.get_mut::<TokenStock>(player).unwrap();
        crate::civilization::functions::return_all_tokens_from_area_to_player(
            &player,
            &area,
            &mut population,
            &mut stock,
            &mut areas,
        );
        world.run_system_once(hide_off_board_pieces).unwrap();
        for t in invaders {
            assert!(world.get::<Sprite>(t).is_none(), "ghost token {t:?}");
        }
    }
}
