//! Cross-checks the three places a city lives: the area's `BuiltCity`, the
//! owner's `PlayerCities` and the owner's `CityTokenStock`.
//!
//! The loss ledger is fed from `PlayerCities`, so it cannot see a city that is
//! on the map but not in its owner's list (or the other way round). This runs
//! on every phase change and warns, once per problem, as `[CITY_AUDIT]`.

use super::{BuiltCity, CityToken, CityTokenStock, PlayerCities, Population};
use crate::GameActivity;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::{
    ClearAllTokens, DestroyCity, ReduceCity, TransferCityTo,
};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Entity, Has, Local, Name, Or, Query, Res, State, With, info, warn};

type Marked = Or<(
    With<DestroyCity>,
    With<ReduceCity>,
    With<TransferCityTo>,
    With<ClearAllTokens>,
)>;

/// Every inconsistency on the board right now.
pub fn city_problems(
    areas: &Query<(Entity, Option<&Name>, &BuiltCity, Option<&Population>)>,
    owners: &Query<(Entity, Option<&Name>, &PlayerCities, &CityTokenStock)>,
    city_tokens: &Query<&CityToken>,
    marked: &Query<(Entity, Option<&Name>, Has<BuiltCity>), Marked>,
    tokens_may_share_areas: bool,
) -> Vec<String> {
    let label = |e: Entity, name: Option<&Name>| {
        name.map_or_else(|| format!("{e:?}"), |n| format!("{n} ({e:?})"))
    };
    let mut problems = Vec::new();
    let mut on_map: HashMap<Entity, (Entity, Entity)> = HashMap::default();
    for (area, name, built, population) in areas {
        let area_label = label(area, name);
        on_map.insert(area, (built.city, built.player));
        match owners.get(built.player) {
            Err(_) => problems.push(format!(
                "{area_label}: city owner {:?} has no PlayerCities/CityTokenStock",
                built.player
            )),
            Ok((_, owner, cities, _)) => {
                if cities.areas_and_cities.get(&area) != Some(&built.city) {
                    problems.push(format!(
                        "{area_label}: city on the map is not in the list of {}",
                        label(built.player, owner)
                    ));
                }
            }
        }
        if city_tokens.get(built.city).map(|t| t.player).ok() != Some(built.player) {
            problems.push(format!(
                "{area_label}: city token does not belong to the city's owner"
            ));
        }
        if !tokens_may_share_areas
            && let Some(n) = population.map(Population::total_population)
            && n > 0
        {
            problems.push(format!(
                "{area_label}: {n} tokens share the area with a city"
            ));
        }
    }
    for (player, name, cities, stock) in owners {
        let player_label = label(player, name);
        for (area, city) in &cities.areas_and_cities {
            if on_map.get(area) != Some(&(*city, player)) {
                problems.push(format!(
                    "{player_label}: listed city in {area:?} is not on the map"
                ));
            }
        }
        let in_stock: HashSet<Entity> = stock.iter().collect();
        let count = stock.city_tokens_in_stock();
        if in_stock.len() != count {
            problems.push(format!(
                "{player_label}: the same city token is in stock twice"
            ));
        }
        if cities
            .areas_and_cities
            .values()
            .any(|c| in_stock.contains(c))
        {
            problems.push(format!(
                "{player_label}: a standing city's token is also in stock"
            ));
        }
        if count + cities.number_of_cities() != stock.max_tokens {
            problems.push(format!(
                "{player_label}: {count} city tokens in stock + {} cities is not {}",
                cities.number_of_cities(),
                stock.max_tokens
            ));
        }
    }
    for (area, name, has_city) in marked {
        problems.push(format!(
            "{}: calamity marker left over ({} city there)",
            label(area, name),
            if has_city { "a" } else { "no" }
        ));
    }
    problems
}

pub fn audit_cities(
    activity: Option<Res<State<GameActivity>>>,
    areas: Query<(Entity, Option<&Name>, &BuiltCity, Option<&Population>)>,
    owners: Query<(Entity, Option<&Name>, &PlayerCities, &CityTokenStock)>,
    city_tokens: Query<&CityToken>,
    marked: Query<(Entity, Option<&Name>, Has<BuiltCity>), Marked>,
    mut known: Local<HashSet<String>>,
) {
    let Some(activity) = activity else {
        return;
    };
    // Tokens that cannot take a city stand in its area until they are removed
    // as surplus.
    let share = matches!(
        activity.get(),
        GameActivity::Movement
            | GameActivity::Conflict
            | GameActivity::BattleReplay
            | GameActivity::CityConstruction
    );
    let problems: HashSet<String> = city_problems(&areas, &owners, &city_tokens, &marked, share)
        .into_iter()
        .collect();
    for new in problems.difference(&known) {
        warn!("[CITY_AUDIT] entering {:?}: {new}", activity.get());
    }
    for gone in known.difference(&problems) {
        info!(
            "[CITY_AUDIT] entering {:?}: resolved: {gone}",
            activity.get()
        );
    }
    *known = problems;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::AvailableFactions;
    use crate::civilization::RoundSummary;
    use crate::civilization::build_city;
    use crate::civilization::components::{Faction, PlayerAreas, TokenStock, Treasury};
    use crate::civilization::concepts::BuildCityCommand;
    use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::clear_leftover_city_markers;
    use crate::civilization::enums::GameFaction;
    use crate::civilization::game_moves::RecalculatePlayerMoves;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    fn problems(world: &mut World) -> Vec<String> {
        world
            .run_system_once(
                |areas: Query<(Entity, Option<&Name>, &BuiltCity, Option<&Population>)>,
                 owners: Query<(Entity, Option<&Name>, &PlayerCities, &CityTokenStock)>,
                 tokens: Query<&CityToken>,
                 marked: Query<(Entity, Option<&Name>, Has<BuiltCity>), Marked>| {
                    city_problems(&areas, &owners, &tokens, &marked, true)
                },
            )
            .unwrap()
    }

    /// A player with nine city tokens, one of them standing in `area`.
    fn player_with_city(world: &mut World, area: Entity, faction: GameFaction) -> Entity {
        let player = world.spawn_empty().id();
        let mut tokens: Vec<Entity> = (0..9)
            .map(|_| world.spawn(CityToken::new(player)).id())
            .collect();
        let city = tokens.pop().unwrap();
        let mut cities = PlayerCities::default();
        cities.build_city_in_area(area, city);
        world.entity_mut(area).insert(BuiltCity::new(city, player));
        world.entity_mut(player).insert((
            cities,
            CityTokenStock::new(9, tokens),
            PlayerAreas::default(),
            TokenStock::new(47, vec![]),
            Treasury::default(),
            Faction::new(faction),
        ));
        player
    }

    #[test]
    fn a_consistent_board_has_no_problems_and_a_phantom_city_is_one() {
        let mut world = World::new();
        let area = world.spawn(Population::new(3)).id();
        player_with_city(&mut world, area, GameFaction::Crete);
        assert_eq!(problems(&mut world), Vec::<String>::new());
        world.entity_mut(area).remove::<BuiltCity>();
        assert_eq!(problems(&mut world).len(), 1);
        world.entity_mut(area).insert(DestroyCity);
        assert_eq!(problems(&mut world).len(), 2, "stale marker");
        world.run_system_once(clear_leftover_city_markers).unwrap();
        assert_eq!(problems(&mut world).len(), 1);
    }

    #[test]
    fn a_city_token_is_in_stock_only_once() {
        let token = Entity::from_raw_u32(7).unwrap();
        let mut stock = CityTokenStock::new(9, vec![]);
        stock.return_token_to_stock(token);
        stock.return_token_to_stock(token);
        assert_eq!(stock.city_tokens_in_stock(), 1);
    }

    /// Rule 25.1: one city per area.
    #[test]
    fn nobody_builds_over_a_standing_city() {
        let mut world = World::new();
        world.init_resource::<Messages<BuildCityCommand>>();
        world.init_resource::<Messages<RecalculatePlayerMoves>>();
        world.init_resource::<RoundSummary>();
        world.init_resource::<AvailableFactions>();
        let contested = world.spawn((Population::new(3), Transform::default())).id();
        let elsewhere = world.spawn((Population::new(3), Transform::default())).id();
        let owner = player_with_city(&mut world, contested, GameFaction::Crete);
        let invader = player_with_city(&mut world, elsewhere, GameFaction::Egypt);
        world
            .resource_mut::<Messages<BuildCityCommand>>()
            .write(BuildCityCommand {
                player: invader,
                area: contested,
            });
        world.run_system_once(build_city).unwrap();
        assert_eq!(world.get::<BuiltCity>(contested).unwrap().player, owner);
        assert_eq!(problems(&mut world), Vec::<String>::new());
    }
}
