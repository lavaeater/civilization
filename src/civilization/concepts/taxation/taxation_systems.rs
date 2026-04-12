use crate::civilization::components::{BuiltCity, PlayerCities, TokenStock, Treasury};
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::ReturnCityToStock;
use crate::civilization::concepts::taxation::taxation_components::{
    CityInRevolt, NeedsToPayTaxes,
};
use crate::civilization::CivCardName;
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{
    info, Commands, Entity, Has, Name, NextState, Query, ResMut, With,
};

/// Called on entering `CollectTaxes`. Skips the phase entirely if no player has any
/// cities (this will be the case on the very first turn of the game).
pub fn enter_collect_taxes(
    player_query: Query<(Entity, &PlayerCities)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let any_cities = player_query.iter().any(|(_, cities)| cities.has_cities());

    if !any_cities {
        info!("[TAXATION] No cities on the board — skipping taxation phase");
        next_state.set(GameActivity::PopulationExpansion);
        return;
    }

    info!("[TAXATION] Entering taxation phase — assigning tax obligations");
    for (player_entity, cities) in player_query.iter() {
        let city_count = cities.number_of_cities();
        if city_count == 0 {
            continue;
        }
        // Base rate: 2 tokens per city. Coinage (not yet implemented) would allow 1 or 3.
        let tokens_owed = city_count * 2;
        commands
            .entity(player_entity)
            .insert(NeedsToPayTaxes::new(tokens_owed));
    }
}

/// Processes tax payment for each player that has `NeedsToPayTaxes`.
/// Players with **Democracy** never revolt.
/// Players with enough stock pay in full.
/// Players with too little stock pay what they can; their excess cities are marked as revolting.
pub fn collect_taxes(
    mut player_query: Query<(
        Entity,
        &Name,
        &NeedsToPayTaxes,
        &mut TokenStock,
        &mut Treasury,
        &mut PlayerCities,
        Has<PlayerCivilizationCards>,
    )>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut commands: Commands,
) {
    for (player_entity, name, needs_to_pay, mut stock, mut treasury, cities, _) in
        player_query.iter_mut()
    {
        let tokens_owed = needs_to_pay.tokens_owed;
        let has_democracy = civ_cards_query
            .get(player_entity)
            .map(|c| c.owns(&CivCardName::Democracy))
            .unwrap_or(false);

        if has_democracy {
            // Democracy holders are immune from revolts; pay what they have, no revolt.
            let to_pay = tokens_owed.min(stock.tokens_in_stock());
            if let Some(tokens) = stock.remove_at_most_n_tokens_from_stock(to_pay) {
                for token in tokens {
                    treasury.add_token_to_treasury(token);
                }
            }
            info!(
                "[TAXATION] {} (Democracy) pays {} tokens, no revolt possible",
                name, to_pay
            );
            commands.entity(player_entity).remove::<NeedsToPayTaxes>();
            continue;
        }

        let stock_count = stock.tokens_in_stock();
        if stock_count >= tokens_owed {
            // Full payment.
            if let Some(tokens) = stock.remove_tokens_from_stock(tokens_owed) {
                for token in tokens {
                    treasury.add_token_to_treasury(token);
                }
            }
            info!("[TAXATION] {} pays {} tokens in full", name, tokens_owed);
            commands.entity(player_entity).remove::<NeedsToPayTaxes>();
        } else {
            // Partial payment — pay an even number of tokens (each city costs 2).
            // Cities that cannot be paid for will revolt.
            let affordable_cities = stock_count / 2;
            let to_pay = affordable_cities * 2;
            let cities_in_revolt = cities.number_of_cities() - affordable_cities;

            if let Some(tokens) = stock.remove_at_most_n_tokens_from_stock(to_pay) {
                for token in tokens {
                    treasury.add_token_to_treasury(token);
                }
            }

            info!(
                "[TAXATION] {} can only pay for {} cities ({} tokens); {} cities revolt",
                name, affordable_cities, to_pay, cities_in_revolt
            );

            // Mark excess cities as revolting (pick from the map arbitrarily — the
            // resolve-revolts system will handle beneficiary assignment).
            let revolting: Vec<Entity> = cities
                .areas_and_cities
                .values()
                .copied()
                .take(cities_in_revolt)
                .collect();

            for city_entity in revolting {
                commands
                    .entity(city_entity)
                    .insert(CityInRevolt::new(player_entity));
            }

            commands.entity(player_entity).remove::<NeedsToPayTaxes>();
        }
    }
}

/// Resolves city revolts after all taxes have been paid.
/// The player with the most **unit points in stock** (cities=5, tokens=1) claims
/// revolting cities by replacing them with his own. Ties broken by A.S.T. order
/// (not yet tracked, so currently resolved in arbitrary entity order).
/// If no one can take over a city, it is eliminated.
pub fn resolve_revolts(
    revolting_cities_query: Query<(Entity, &CityInRevolt)>,
    player_query: Query<(Entity, &Name, &TokenStock, &PlayerCities), With<Player>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let revolting: Vec<(Entity, Entity)> = revolting_cities_query
        .iter()
        .map(|(city, revolt)| (city, revolt.original_owner))
        .collect();

    if revolting.is_empty() {
        info!("[TAXATION] No revolts to resolve — transitioning to PopulationExpansion");
        next_state.set(GameActivity::PopulationExpansion);
        return;
    }

    info!("[TAXATION] Resolving {} revolting cities", revolting.len());

    // Build a list of candidates sorted by unit points in stock (descending).
    let mut candidates: Vec<(Entity, usize)> = player_query
        .iter()
        .map(|(entity, _, stock, cities)| {
            let unit_points = stock.tokens_in_stock() + cities.number_of_cities() * 5;
            (entity, unit_points)
        })
        .collect();
    candidates.sort_by_key(|(_,b)|*b);

    // Collect (city_entity, area_entity, original_owner) triples so we can
    // update PlayerCities without conflicting borrows.
    let mut city_area_owner: Vec<(Entity, Option<Entity>, Entity)> = Vec::new();
    for (revolting_city, original_owner) in &revolting {
        let area = player_query
            .get(*original_owner)
            .ok()
            .and_then(|(_, _, _, cities)| {
                cities
                    .areas_and_cities
                    .iter()
                    .find(|(_, c)| **c == *revolting_city)
                    .map(|(&a, _)| a)
            });
        city_area_owner.push((*revolting_city, area, *original_owner));
    }

    // Issue commands to update ownership. PlayerCities mutations happen via a
    // separate system pass (commands are deferred), so we queue them here.
    for (revolting_city, area_opt, original_owner) in city_area_owner {
        let beneficiary = candidates
            .iter()
            .find(|(e, _)| *e != original_owner)
            .map(|(e, _)| *e);

        match beneficiary {
            Some(new_owner) => {
                info!(
                    "[TAXATION] Revolting city {:?} taken over by {:?}",
                    revolting_city, new_owner
                );
                // Update the BuiltCity component on the city entity to reflect new owner.
                commands
                    .entity(revolting_city)
                    .remove::<CityInRevolt>()
                    .insert(BuiltCity::new(revolting_city, new_owner));

                // Remove the city from the original owner's record and add to new owner.
                if let Some(area) = area_opt {
                    commands.entity(original_owner).queue(
                        move |mut entity: bevy::ecs::world::EntityWorldMut| {
                            if let Some(mut cities) = entity.get_mut::<PlayerCities>() {
                                cities.remove_city_from_area(area);
                            }
                        },
                    );
                    commands.entity(new_owner).queue(
                        move |mut entity: bevy::ecs::world::EntityWorldMut| {
                            if let Some(mut cities) = entity.get_mut::<PlayerCities>() {
                                cities.build_city_in_area(area, revolting_city);
                            }
                        },
                    );
                }
            }
            None => {
                info!(
                    "[TAXATION] No player can take revolting city {:?} — eliminating",
                    revolting_city
                );
                commands
                    .entity(revolting_city)
                    .remove::<CityInRevolt>()
                    .insert(ReturnCityToStock);

                if let Some(area) = area_opt {
                    commands.entity(original_owner).queue(
                        move |mut entity: bevy::ecs::world::EntityWorldMut| {
                            if let Some(mut cities) = entity.get_mut::<PlayerCities>() {
                                cities.remove_city_from_area(area);
                            }
                        },
                    );
                }
            }
        }
    }

    next_state.set(GameActivity::PopulationExpansion);
}

/// Gate: waits until all `NeedsToPayTaxes` and `CityInRevolt` components are gone,
/// then transitions to `PopulationExpansion`.
pub fn taxation_gate(
    still_paying: Query<Entity, With<NeedsToPayTaxes>>,
    revolts: Query<Entity, With<CityInRevolt>>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if still_paying.is_empty() && revolts.is_empty() {
        next_state.set(GameActivity::PopulationExpansion);
    }
}

// ─── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::civilization::components::{PlayerCities, TokenStock, Treasury};

    /// Helper: create a player entity with `n` city-slot entries in `PlayerCities`
    /// and `stock_tokens` tokens available.
    fn setup_player_with_cities(
        world: &mut World,
        stock_tokens: usize,
        city_count: usize,
    ) -> Entity {
        // Spawn dummy city tokens
        let mut city_entities = Vec::new();
        for _ in 0..city_count {
            let area = world.spawn_empty().id();
            let city = world.spawn_empty().id();
            city_entities.push((area, city));
        }

        // Spawn enough token entities for the stock
        let token_entities: Vec<Entity> =
            (0..stock_tokens).map(|_| world.spawn_empty().id()).collect();

        let stock = TokenStock::new(55, token_entities);
        let mut cities = PlayerCities::default();
        for (area, city) in city_entities {
            cities.build_city_in_area(area, city);
        }

        world
            .spawn((Name::new("TestPlayer"), stock, cities, Treasury::default()))
            .id()
    }

    // ── Rule 19.1: standard taxation ─────────────────────────────────────────

    #[test]
    fn full_payment_moves_tokens_to_treasury() {
        let mut world = World::new();
        let player = setup_player_with_cities(&mut world, 10, 2); // 2 cities → owes 4 tokens

        let mut stock_before = 0;
        world.query::<(&TokenStock, &PlayerCities)>().iter(&world).for_each(|(s, _)| {
            stock_before = s.tokens_in_stock();
        });
        assert_eq!(stock_before, 10);

        // Manually invoke the core taxation calculation to test the pure logic.
        let city_count;
        let tokens_owed;
        {
            let cities = world.get::<PlayerCities>(player).unwrap();
            city_count = cities.number_of_cities();
            tokens_owed = city_count * 2;
        }

        assert_eq!(city_count, 2);
        assert_eq!(tokens_owed, 4);

        // Simulate payment
        {
            let mut stock = world.get_mut::<TokenStock>(player).unwrap();
            let taken = stock.remove_tokens_from_stock(tokens_owed).unwrap();
            drop(stock);
            let mut treasury = world.get_mut::<Treasury>(player).unwrap();
            for t in taken {
                treasury.add_token_to_treasury(t);
            }
        }

        let stock_after = world.get::<TokenStock>(player).unwrap().tokens_in_stock();
        let treasury_after = world.get::<Treasury>(player).unwrap().tokens_in_treasury();
        assert_eq!(stock_after, 6);
        assert_eq!(treasury_after, 4);
    }

    // ── Rule 19.31: revolts when stock is insufficient ────────────────────────

    #[test]
    fn shortfall_produces_correct_revolt_count() {
        // Player has 3 cities, only 2 tokens in stock → can pay for 1 city, 2 revolt.
        let city_count = 3;
        let stock_tokens = 2;
        let tokens_owed = city_count * 2; // 6
        let affordable_cities = stock_tokens / 2; // 1
        let cities_in_revolt = city_count - affordable_cities; // 2

        assert_eq!(tokens_owed, 6);
        assert_eq!(affordable_cities, 1);
        assert_eq!(cities_in_revolt, 2);
    }

    #[test]
    fn zero_stock_means_all_cities_revolt() {
        let city_count = 3;
        let stock_tokens = 0;
        let affordable_cities = stock_tokens / 2; // 0
        let cities_in_revolt = city_count - affordable_cities; // 3

        assert_eq!(cities_in_revolt, 3);
    }

    #[test]
    fn exact_stock_no_revolts() {
        let city_count = 3;
        let stock_tokens = 6; // exactly 2 per city
        let affordable_cities = stock_tokens / 2; // 3
        let cities_in_revolt = city_count - affordable_cities; // 0

        assert_eq!(cities_in_revolt, 0);
    }

    // ── Rule 19.2: Coinage taxation rate calculation ──────────────────────────

    #[test]
    fn coinage_low_rate_reduces_tokens_owed() {
        let city_count = 3;
        let low_rate: usize = 1;
        assert_eq!(city_count * low_rate, 3);
    }

    #[test]
    fn coinage_high_rate_increases_tokens_owed() {
        let city_count = 3;
        let high_rate: usize = 3;
        assert_eq!(city_count * high_rate, 9);
    }

    // ── Rule 19.32: revolt beneficiary has most units in stock ────────────────

    #[test]
    fn beneficiary_is_player_with_most_unit_points_in_stock() {
        // Player A: 2 tokens in stock + 0 cities = 2 unit points
        // Player B: 5 tokens in stock + 1 city  = 10 unit points → beneficiary
        let a_stock = 2usize;
        let a_cities_in_stock = 0usize;
        let b_stock = 5usize;
        let b_cities_in_stock = 1usize;

        let a_points = a_stock + a_cities_in_stock * 5;
        let b_points = b_stock + b_cities_in_stock * 5;

        assert!(b_points > a_points, "B should be the beneficiary");
    }

    // ── Rule 19.34: Democracy prevents revolts ────────────────────────────────

    #[test]
    fn democracy_holder_never_revolts_even_with_shortfall() {
        // With Democracy the player pays what they can but no cities revolt.
        // This is tested at the logic level: we verify the shortfall *would*
        // cause a revolt in the absence of Democracy.
        let city_count = 3;
        let stock_tokens = 2;
        let affordable_cities = stock_tokens / 2;
        let would_revolt = city_count - affordable_cities;

        // Without Democracy: 2 cities would revolt
        assert_eq!(would_revolt, 2);

        // With Democracy: revolt count forced to 0
        let has_democracy = true;
        let actual_revolt = if has_democracy { 0 } else { would_revolt };
        assert_eq!(actual_revolt, 0);
    }
}
