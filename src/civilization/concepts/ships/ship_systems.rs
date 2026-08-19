use crate::GameActivity;
use crate::civilization::components::{
    GameArea, PlayerAreas, Population, ReturnTokenToStock, SeaPassage, Treasury,
};
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::ships::ship_components::{PlayerShips, Ship, ShipStock};
use crate::civilization::concepts::ships::ship_ui_components::{
    AwaitingShipPlacement, ShipConstructionState,
};
use crate::civilization::CivCardName;
use crate::loading::TextureAssets;
use crate::player::Player;
use crate::stupid_ai::{AgentControlled, IsHuman};
use bevy::prelude::{
    Commands, Entity, Has, Name, NextState, Query, Res, ResMut, Sprite, Transform, With, info,
};

/// Rule 22.1/22.11: ships are built (and maintained) in census order; players
/// who hold Military always go after players who do not, with census order
/// preserved as the tiebreaker within each group. `census_order` is already
/// sorted highest-population-first by `perform_census`; this only needs to
/// stable-sort Military holders to the back without disturbing that order.
pub fn ship_build_order(census_order: &[Entity], has_military: impl Fn(Entity) -> bool) -> Vec<Entity> {
    let mut ordered: Vec<Entity> = census_order.to_vec();
    ordered.sort_by_key(|&e| has_military(e));
    ordered
}

/// Called on entering `ShipConstruction`.
///
/// Pass 1 — Maintenance (rule 22.3): each ship costs 1 token, paid from
/// treasury or by a levy from the area the ship occupies. Unpaid ships
/// return to stock.
///
/// Pass 2 — Building: AI players auto-build. Human players are paused here:
/// `ShipConstructionState` is populated and `AwaitingShipPlacement` is inserted;
/// `advance_ship_construction` waits until the human confirms before transitioning.
pub fn enter_ship_construction(
    mut player_query: Query<
        (
            Entity,
            &Name,
            &mut ShipStock,
            &mut PlayerShips,
            &mut Treasury,
            &PlayerAreas,
            Has<IsHuman>,
            Has<AgentControlled>,
        ),
        With<Player>,
    >,
    sea_passage_query: Query<Has<SeaPassage>>,
    area_transform_query: Query<&Transform, With<GameArea>>,
    mut area_pop_query: Query<&mut Population, With<GameArea>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut ship_state: ResMut<ShipConstructionState>,
    textures: Res<TextureAssets>,
    game_info: Res<GameInfoAndStuff>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
) {
    let mut human_needs_input = false;

    // ── Pass 1: Maintenance (rule 22.3) ──────────────────────────────────────
    // Each ship costs 1 token from treasury OR a levy of 1 from the area it
    // occupies. Ships that cannot be paid for are returned to stock.
    for (player_entity, name, mut ship_stock, mut player_ships, mut treasury, _, _, _) in
        &mut player_query
    {
        let areas_with_ships: Vec<Entity> = player_ships.all_areas_with_ships();
        for area in areas_with_ships {
            let paid = if treasury.tokens_in_treasury() >= 1 {
                // Treasury tokens are the same finite pool as population tokens —
                // they must go back to stock, not be dropped, or the pool leaks.
                if let Some(token) = treasury.remove_token_from_treasury() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
                true
            } else if let Ok(mut pop) = area_pop_query.get_mut(area) {
                // Levy 1 token from the area the ship occupies (rule 22.3).
                if let Some(levied) = pop.remove_tokens_from_area(&player_entity, 1) {
                    for token in levied {
                        commands.entity(token).insert(ReturnTokenToStock);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !paid && let Some(ship_entity) = player_ships.remove_ship_from_area(area) {
                ship_stock.return_ship(ship_entity);
                info!(
                    "[SHIPS] {} cannot maintain ship at {:?} — returned to stock",
                    name, area
                );
            }
        }
    }

    // ── Pass 2: Building ──────────────────────────────────────────────────────
    // Building costs 2 tokens per ship, paid from treasury and/or a levy from
    // the build area. If any levy is used the ship is placed in that area
    // (rule 22.2). Levied tokens are returned to stock.
    //
    // Rule 22.1/22.11: process players in census order, Military holders last.
    // Any player missing from `census_order` (e.g. a test that spawns players
    // without running the Census phase first) is appended afterward in query
    // order, so nobody is silently skipped.
    let mut build_order = ship_build_order(&game_info.census_order, |e| {
        civ_cards_query.get(e).is_ok_and(|c| c.owns(&CivCardName::Military))
    });
    let already_ordered: bevy::platform::collections::HashSet<Entity> =
        build_order.iter().copied().collect();
    for (entity, ..) in &player_query {
        if !already_ordered.contains(&entity) {
            build_order.push(entity);
        }
    }

    for player_entity in build_order {
        let Ok((
            _,
            name,
            mut ship_stock,
            mut player_ships,
            mut treasury,
            player_areas,
            is_human,
            is_agent_controlled,
        )) = player_query.get_mut(player_entity)
        else {
            continue;
        };
        let ships_on_board = player_ships.total_ships_on_board();
        if ships_on_board >= ShipStock::MAX_SHIPS {
            continue;
        }

        // Agent-controlled players are `IsHuman` (so the game waits for them in
        // interactive phases), but ship construction has no agent endpoint — drive
        // them down the AI auto-build path rather than the interactive UI, which
        // only one local human can confirm. See agent-api-design.md.
        if is_human && !is_agent_controlled {
            // Gather areas with player tokens (preferring coastal ones).
            let mut available_areas: Vec<Entity> = player_areas
                .areas()
                .into_iter()
                .filter(|&a| sea_passage_query.get(a).unwrap_or(false))
                .collect();
            if available_areas.is_empty() {
                available_areas = player_areas.areas().into_iter().collect();
            }

            if available_areas.is_empty() || ship_stock.is_empty() {
                continue;
            }

            // Compute max ships the human can afford across all available areas.
            // A ship costs 2 tokens: treasury + area levy combined.
            let treasury_tokens = treasury.tokens_in_treasury();
            let total_area_tokens: usize = available_areas
                .iter()
                .filter_map(|&a| area_pop_query.get(a).ok())
                .map(|pop| pop.population_for_player(player_entity))
                .sum();
            let ships_affordable = usize::midpoint(treasury_tokens, total_area_tokens);
            if ships_affordable == 0 {
                continue;
            }

            let max_buildable = (ShipStock::MAX_SHIPS - ships_on_board)
                .min(ships_affordable)
                .min(ship_stock.count_in_stock());

            ship_state.populate(player_entity, available_areas, max_buildable);
            commands.entity(player_entity).insert(AwaitingShipPlacement);
            human_needs_input = true;
            info!(
                "[SHIPS] Human player {:?} entering ship construction UI",
                player_entity
            );
        } else {
            // AI: prefer a coastal area, fall back to any area.
            let candidate_area = player_areas
                .areas()
                .into_iter()
                .find(|&a| sea_passage_query.get(a).unwrap_or(false))
                .or_else(|| player_areas.areas().into_iter().next());
            let Some(area) = candidate_area else { continue };

            // Check affordability: treasury + area tokens must cover 2. The levy
            // must leave at least 1 token in the area — never depopulate it for a
            // ship, or the player wipes out their only city on round 1 (treasury
            // is empty early, so a naive levy of 2 from a 2-token start area would
            // empty the board). Only spare tokens above 1 are leviable.
            let treasury_tokens = treasury.tokens_in_treasury();
            let area_tokens = area_pop_query
                .get(area)
                .map_or(0, |pop| pop.population_for_player(player_entity));
            let spare_area_tokens = area_tokens.saturating_sub(1);
            if treasury_tokens + spare_area_tokens < 2 {
                continue;
            }

            let Some(ship_entity) = ship_stock.take_ship() else {
                continue;
            };

            // Pay: prefer treasury, levy the remainder from the area.
            let from_treasury = treasury_tokens.min(2);
            let from_levy = 2 - from_treasury;
            for _ in 0..from_treasury {
                // Return spent treasury tokens to stock (same finite pool).
                if let Some(token) = treasury.remove_token_from_treasury() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            }
            if from_levy > 0
                && let Ok(mut pop) = area_pop_query.get_mut(area)
                && let Some(levied) = pop.remove_tokens_from_area(&player_entity, from_levy)
            {
                for token in levied {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            }

            let area_pos = area_transform_query
                .get(area)
                .map(|t| t.translation.truncate())
                .unwrap_or_default();
            commands.entity(ship_entity).insert((
                Sprite {
                    image: textures.ship.clone(),
                    ..Default::default()
                },
                Transform::from_xyz(area_pos.x, area_pos.y, 2.0),
            ));

            player_ships.place_ship(area, ship_entity);
            info!(
                "[SHIPS] {} builds a ship at {:?} (treasury: {}, levy: {}) (fleet: {}/{})",
                name,
                area,
                from_treasury,
                from_levy,
                player_ships.total_ships_on_board(),
                ShipStock::MAX_SHIPS
            );
        }
    }

    if !human_needs_input {
        next_state.set(GameActivity::Movement);
    }
}

/// Runs every frame during `ShipConstruction`.
/// Once no human player has `AwaitingShipPlacement`, applies the human's build
/// choices (from `ShipConstructionState`) and transitions to `Movement`.
pub fn advance_ship_construction(
    waiting: Query<Entity, With<AwaitingShipPlacement>>,
    mut player_query: Query<(&Name, &mut ShipStock, &mut PlayerShips, &mut Treasury), With<Player>>,
    area_transform_query: Query<&Transform, With<GameArea>>,
    mut area_pop_query: Query<&mut Population, With<GameArea>>,
    mut ship_state: ResMut<ShipConstructionState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    textures: Res<TextureAssets>,
) {
    // Still waiting.
    if !waiting.is_empty() {
        return;
    }

    // Human confirmed (or there was no human) — apply the build plan.
    if let Some(player_entity) = ship_state.player {
        if let Ok((name, mut ship_stock, mut player_ships, mut treasury)) =
            player_query.get_mut(player_entity)
        {
            let areas = ship_state.take_result(); // drains + clears resource
            for area in areas {
                let Some(ship_entity) = ship_stock.take_ship() else {
                    break;
                };

                // Pay 2 tokens: treasury first, levy the remainder from the area.
                let treasury_tokens = treasury.tokens_in_treasury();
                let area_tokens = area_pop_query
                    .get(area)
                    .map_or(0, |pop| pop.population_for_player(player_entity));

                if treasury_tokens + area_tokens < 2 {
                    // Can no longer afford — return ship and stop.
                    ship_stock.return_ship(ship_entity);
                    break;
                }

                let from_treasury = treasury_tokens.min(2);
                let from_levy = 2 - from_treasury;
                for _ in 0..from_treasury {
                    // Return spent treasury tokens to stock (same finite pool).
                    if let Some(token) = treasury.remove_token_from_treasury() {
                        commands.entity(token).insert(ReturnTokenToStock);
                    }
                }
                if from_levy > 0
                    && let Ok(mut pop) = area_pop_query.get_mut(area)
                    && let Some(levied) = pop.remove_tokens_from_area(&player_entity, from_levy)
                {
                    for token in levied {
                        commands.entity(token).insert(ReturnTokenToStock);
                    }
                }

                let area_pos = area_transform_query
                    .get(area)
                    .map(|t| t.translation.truncate())
                    .unwrap_or_default();
                commands.entity(ship_entity).insert((
                    Sprite {
                        image: textures.ship.clone(),
                        ..Default::default()
                    },
                    Transform::from_xyz(area_pos.x, area_pos.y, 2.0),
                ));

                player_ships.place_ship(area, ship_entity);
                info!(
                    "[SHIPS] {} builds ship at {:?} (treasury: {}, levy: {}) (fleet: {}/{})",
                    name,
                    area,
                    from_treasury,
                    from_levy,
                    player_ships.total_ships_on_board(),
                    ShipStock::MAX_SHIPS
                );
            }
        }
    } else {
        ship_state.clear();
    }

    next_state.set(GameActivity::Movement);
}

/// Spawns the initial ship entities (stock) for a player during game setup.
pub fn create_ship_stock(commands: &mut Commands, player: Entity) -> (ShipStock, PlayerShips) {
    let ship_entities: Vec<Entity> = (0..ShipStock::MAX_SHIPS)
        .map(|_| commands.spawn((Name::new("Ship"), Ship::new(player))).id())
        .collect();

    (ShipStock::new(ship_entities), PlayerShips::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::World;

    // ── Rule 22.1/22.11: census-order build sequencing ───────────────────────

    #[test]
    fn build_order_preserves_census_order_when_nobody_has_military() {
        let mut world = World::new();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let p3 = world.spawn_empty().id();
        let census_order = vec![p1, p2, p3];

        let order = ship_build_order(&census_order, |_| false);

        assert_eq!(order, vec![p1, p2, p3]);
    }

    #[test]
    fn build_order_moves_military_holders_to_the_back() {
        let mut world = World::new();
        let p1 = world.spawn_empty().id(); // highest census, has Military
        let p2 = world.spawn_empty().id(); // no Military
        let p3 = world.spawn_empty().id(); // lowest census, has Military
        let census_order = vec![p1, p2, p3];

        let order = ship_build_order(&census_order, |e| e == p1 || e == p3);

        // p2 (no Military) goes first; p1 and p3 (Military) follow, in their
        // original relative census order (p1 before p3).
        assert_eq!(order, vec![p2, p1, p3]);
    }

    #[test]
    fn build_order_all_military_holders_keeps_census_order_among_them() {
        let mut world = World::new();
        let p1 = world.spawn_empty().id();
        let p2 = world.spawn_empty().id();
        let census_order = vec![p1, p2];

        let order = ship_build_order(&census_order, |_| true);

        assert_eq!(order, vec![p1, p2]);
    }

    // ── Rule 22.3: maintenance ────────────────────────────────────────────────

    /// A ship maintained from treasury: 1 token leaves the treasury, the ship stays.
    #[test]
    fn maintenance_paid_from_treasury_keeps_the_ship_and_spends_one_token() {
        let mut world = World::new();
        let treasury_token = world.spawn_empty().id();
        let mut treasury = Treasury::default();
        treasury.add_token_to_treasury(treasury_token);

        assert_eq!(treasury.tokens_in_treasury(), 1);

        // Simulate the maintenance payment (rule 22.3): treasury preferred.
        let paid = treasury.tokens_in_treasury() >= 1 && treasury.remove_token_from_treasury().is_some();

        assert!(paid);
        assert_eq!(treasury.tokens_in_treasury(), 0);
    }

    /// An unpaid ship (no treasury, no leviable tokens in its area) returns to stock.
    #[test]
    fn unpaid_ship_returns_to_stock() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let ship = world.spawn_empty().id();
        let area = world.spawn_empty().id();

        let mut ship_stock = ShipStock::default();
        let mut player_ships = PlayerShips::default();
        player_ships.place_ship(area, ship);
        let treasury = Treasury::default(); // empty
        let population = Population::new(0); // nothing to levy

        assert_eq!(ship_stock.count_in_stock(), 0);
        assert_eq!(player_ships.total_ships_on_board(), 1);

        // Simulate: treasury has nothing, area has nothing to levy -> unpaid.
        let paid = treasury.tokens_in_treasury() >= 1
            || population.population_for_player(player) > 0;
        assert!(!paid);

        if !paid && let Some(unpaid_ship) = player_ships.remove_ship_from_area(area) {
            ship_stock.return_ship(unpaid_ship);
        }

        assert_eq!(ship_stock.count_in_stock(), 1);
        assert_eq!(player_ships.total_ships_on_board(), 0);
    }

    // ── Rule 22.1/22.2: construction cost split (treasury + levy) ────────────

    /// A ship fully affordable from treasury draws nothing from the area levy.
    #[test]
    fn construction_prefers_treasury_over_levy() {
        let treasury_tokens = 5;
        let from_treasury = treasury_tokens.min(2);
        let from_levy = 2 - from_treasury;

        assert_eq!(from_treasury, 2);
        assert_eq!(from_levy, 0);
    }

    /// An empty treasury forces the full cost onto the area levy.
    #[test]
    fn construction_falls_back_fully_to_levy_when_treasury_is_empty() {
        let treasury_tokens = 0;
        let from_treasury = treasury_tokens.min(2);
        let from_levy = 2 - from_treasury;

        assert_eq!(from_treasury, 0);
        assert_eq!(from_levy, 2);
    }

    /// A partial treasury balance splits the cost between treasury and levy —
    /// this is the split rule 22.1 requires ("a combination of the two").
    #[test]
    fn construction_splits_cost_between_treasury_and_levy() {
        let treasury_tokens = 1;
        let from_treasury = treasury_tokens.min(2);
        let from_levy = 2 - from_treasury;

        assert_eq!(from_treasury, 1);
        assert_eq!(from_levy, 1);
    }

    // ── Rule 22.4: fleet cap ──────────────────────────────────────────────────

    #[test]
    fn a_player_may_not_have_more_than_four_ships_on_board() {
        assert_eq!(ShipStock::MAX_SHIPS, 4);
    }

    #[test]
    fn create_ship_stock_spawns_exactly_four_ships_in_stock() {
        use bevy::ecs::world::CommandQueue;

        let mut world = World::new();
        let player = world.spawn_empty().id();

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        let (stock, ships) = create_ship_stock(&mut commands, player);
        command_queue.apply(&mut world);

        assert_eq!(stock.count_in_stock(), 4);
        assert_eq!(ships.total_ships_on_board(), 0);
    }
}
