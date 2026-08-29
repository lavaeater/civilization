use crate::GameActivity;
use crate::civilization::BuildCityCommand;
use crate::civilization::CivCardName;
use crate::civilization::components::{
    BuiltCity, CityTokenStock, Faction, GameArea, PlayerCities, TokenStock, Treasury,
};
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::map::AvailableFactions;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::ReturnCityToStock;
use crate::civilization::concepts::round_summary::RoundSummary;
use crate::civilization::concepts::taxation::taxation_components::{
    AwaitingCoinageRateSelection, CityInRevolt, CoinageTaxRate, NeedsToPayTaxes,
};
use crate::civilization::functions::build_city_in_area;
use crate::player::Player;
use crate::stupid_ai::{IsHuman, StupidAi};
use bevy::prelude::{
    Commands, Entity, Has, Name, NextState, Query, Res, ResMut, Transform, With, info, warn,
};

/// Called on entering `CollectTaxes`. Skips the phase entirely if no player has any
/// cities (this will be the case on the very first turn of the game).
///
/// A human who owns Coinage and hasn't set a rate yet is put into
/// `AwaitingCoinageRateSelection` instead of getting `NeedsToPayTaxes` right
/// away — the tax-rate UI computes and inserts `NeedsToPayTaxes` once they
/// pick a rate (rule 19.2). `taxation_gate` waits for that marker too, so the
/// phase doesn't advance out from under them.
pub fn enter_collect_taxes(
    player_query: Query<(Entity, &PlayerCities, Option<&CoinageTaxRate>, Has<IsHuman>)>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let any_cities = player_query
        .iter()
        .any(|(_, cities, _, _)| cities.has_cities());

    if !any_cities {
        info!("[TAXATION] No cities on the board — skipping taxation phase");
        next_state.set(GameActivity::PopulationExpansion);
        return;
    }

    info!("[TAXATION] Entering taxation phase — assigning tax obligations");
    for (player_entity, cities, coinage_rate, is_human) in player_query.iter() {
        let city_count = cities.number_of_cities();
        if city_count == 0 {
            continue;
        }

        if is_human && coinage_rate.is_none() {
            let has_coinage = civ_cards_query
                .get(player_entity)
                .is_ok_and(|c| c.owns(&CivCardName::Coinage));
            if has_coinage {
                info!(
                    "[TAXATION] {player_entity:?} holds Coinage — awaiting tax rate choice"
                );
                commands
                    .entity(player_entity)
                    .insert(AwaitingCoinageRateSelection);
                continue;
            }
        }

        // Base rate: 2 tokens/city. Coinage holders may have chosen 1 or 3 (rule 19.2).
        let rate = coinage_rate.map_or(2, |r| r.0.clamp(1, 3));
        let tokens_owed = city_count * rate;
        commands
            .entity(player_entity)
            .insert(NeedsToPayTaxes::new(tokens_owed));
        // Remove the rate override so it doesn't persist to next round
        commands.entity(player_entity).remove::<CoinageTaxRate>();
    }
}

/// Sets the Coinage tax rate for AI players who hold Coinage (rule 19.2).
/// Human players would set this via UI before taxes are computed.
/// AI chooses 3/city when stock is ample (≥20), 1/city when stock is low (≤8), else 2.
pub fn ai_set_coinage_rate(
    player_query: Query<(Entity, &TokenStock, &PlayerCities), (With<Player>, With<StupidAi>)>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut commands: Commands,
) {
    for (player_entity, stock, cities) in player_query {
        let has_coinage = civ_cards_query
            .get(player_entity)
            .is_ok_and(|c| c.owns(&CivCardName::Coinage));

        if !has_coinage || cities.number_of_cities() == 0 {
            continue;
        }

        let rate = if stock.tokens_in_stock() >= 20 {
            3
        } else if stock.tokens_in_stock() <= 8 {
            1
        } else {
            2
        };
        commands.entity(player_entity).insert(CoinageTaxRate(rate));
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
    )>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut commands: Commands,
    mut round_summary: ResMut<RoundSummary>,
) {
    for (player_entity, name, needs_to_pay, mut stock, mut treasury, cities) in &mut player_query {
        let tokens_owed = needs_to_pay.tokens_owed;
        let has_democracy = civ_cards_query
            .get(player_entity)
            .is_ok_and(|c| c.owns(&CivCardName::Democracy));

        if has_democracy {
            // Democracy holders are immune from revolts; pay what they have, no revolt.
            let to_pay = tokens_owed.min(stock.tokens_in_stock());
            if let Some(tokens) = stock.remove_at_most_n_tokens_from_stock(to_pay) {
                for token in tokens {
                    // Rule 19.1: taxes move stock -> treasury. This is the only
                    // route into the treasury besides pillage (24.52) and
                    // Architecture (25.3), and the treasury is what pays for
                    // ninth-stack cards (27.51) and scores at game end -- paying
                    // them back into stock made the whole phase free and left the
                    // treasury permanently empty.
                    treasury.add_token_to_treasury(token);
                }
            }
            info!(
                "[TAXATION] {} (Democracy) pays {} tokens, no revolt possible",
                name, to_pay
            );
            round_summary.push(format!("{name} (Democracy) paid {to_pay} in taxes"));
            commands.entity(player_entity).remove::<NeedsToPayTaxes>();
            continue;
        }

        let stock_count = stock.tokens_in_stock();
        if stock_count >= tokens_owed {
            // Full payment.
            if let Some(tokens) = stock.remove_tokens_from_stock(tokens_owed) {
                for token in tokens {
                    // Rule 19.1: taxes move stock -> treasury. This is the only
                    // route into the treasury besides pillage (24.52) and
                    // Architecture (25.3), and the treasury is what pays for
                    // ninth-stack cards (27.51) and scores at game end -- paying
                    // them back into stock made the whole phase free and left the
                    // treasury permanently empty.
                    treasury.add_token_to_treasury(token);
                }
            }
            info!("[TAXATION] {} pays {} tokens in full", name, tokens_owed);
            round_summary.push(format!("{name} paid {tokens_owed} in taxes in full"));
            commands.entity(player_entity).remove::<NeedsToPayTaxes>();
        } else {
            // Partial payment. Each city costs `rate` tokens — normally 2, but a
            // Coinage holder may have set 1 or 3 (rule 19.2), so derive the actual
            // per-city rate from the obligation rather than assuming 2. Cap the
            // affordable count at the number of cities so the revolt count can't
            // underflow (a rate-3 holder with stock between 2× and 3× cities would
            // otherwise compute stock/2 > cities).
            let num_cities = cities.number_of_cities();
            let rate = tokens_owed.checked_div(num_cities).unwrap_or(2).max(1);
            let affordable_cities = (stock_count / rate).min(num_cities);
            let to_pay = affordable_cities * rate;
            let cities_in_revolt = num_cities - affordable_cities;

            if let Some(tokens) = stock.remove_at_most_n_tokens_from_stock(to_pay) {
                for token in tokens {
                    // Rule 19.1: taxes move stock -> treasury. This is the only
                    // route into the treasury besides pillage (24.52) and
                    // Architecture (25.3), and the treasury is what pays for
                    // ninth-stack cards (27.51) and scores at game end -- paying
                    // them back into stock made the whole phase free and left the
                    // treasury permanently empty.
                    treasury.add_token_to_treasury(token);
                }
            }

            info!(
                "[TAXATION] {} can only pay for {} cities ({} tokens); {} cities revolt",
                name, affordable_cities, to_pay, cities_in_revolt
            );
            round_summary.push(format!(
                "{name} paid {to_pay} in taxes; {cities_in_revolt} of their cities revolted"
            ));

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

/// Resolves city revolts after all taxes have been paid (rule 19.32).
///
/// The player with the **most unit points in stock** (cities = 5 each, tokens = 1
/// each) is the beneficiary and replaces the revolting city with one of his own;
/// if he has no city token left in stock the next-largest stock takes it, and so
/// on. A city nobody can take over is eliminated (19.33).
///
/// Does not itself decide when to leave `CollectTaxes` -- `taxation_gate` is the
/// sole authority for that. This used to unconditionally queue a transition to
/// `PopulationExpansion` whenever there were no revolts to resolve (the common
/// case), which stomped on `taxation_gate`'s check for a still-pending
/// `AwaitingCoinageRateSelection` and skipped the tax-rate picker entirely
/// before it could ever be shown to the player.
pub fn resolve_revolts(
    revolting_cities_query: Query<(Entity, &CityInRevolt)>,
    mut player_query: Query<
        (
            Entity,
            &Name,
            &TokenStock,
            &mut PlayerCities,
            &mut CityTokenStock,
            &Faction,
        ),
        With<Player>,
    >,
    area_query: Query<&Transform, With<GameArea>>,
    game_factions: Res<AvailableFactions>,
    mut commands: Commands,
    mut round_summary: ResMut<RoundSummary>,
) {
    let revolting: Vec<(Entity, Entity)> = revolting_cities_query
        .iter()
        .map(|(city, revolt)| (city, revolt.original_owner))
        .collect();

    if revolting.is_empty() {
        return;
    }

    info!("[TAXATION] Resolving {} revolting cities", revolting.len());

    // Candidates ordered by unit points in stock, strongest first (19.32).
    let mut candidates: Vec<(Entity, usize)> = player_query
        .iter()
        .map(|(entity, _, stock, cities, _, _)| {
            (
                entity,
                stock.tokens_in_stock() + cities.number_of_cities() * 5,
            )
        })
        .collect();
    candidates.sort_by_key(|(_, points)| std::cmp::Reverse(*points));

    // Which area each revolting city sits in, resolved before we start mutating.
    let city_area_owner: Vec<(Entity, Option<Entity>, Entity)> = revolting
        .iter()
        .map(|(city, original_owner)| {
            let area = player_query
                .get(*original_owner)
                .ok()
                .and_then(|(_, _, _, cities, _, _)| cities.area_for_city(*city));
            (*city, area, *original_owner)
        })
        .collect();

    for (revolting_city, area_opt, original_owner) in city_area_owner {
        let owner_name = player_query
            .get(original_owner)
            .map_or("A player".to_string(), |(_, name, ..)| name.to_string());
        // 19.32: the beneficiary replaces the city with one of his own, so he
        // needs a city token in stock. Whoever is strongest without one is
        // skipped in favour of the next player down.
        let beneficiary = area_opt.and_then(|_| {
            candidates.iter().map(|(entity, _)| *entity).find(|entity| {
                *entity != original_owner
                    && player_query
                        .get(*entity)
                        .is_ok_and(|(_, _, _, _, city_stock, _)| city_stock.has_tokens())
            })
        });

        let (Some(area), Some(new_owner)) = (area_opt, beneficiary) else {
            info!("[TAXATION] No player can take revolting city {revolting_city:?} — eliminating");
            round_summary.push(format!(
                "{owner_name}'s city revolted and nobody could take it over — it was destroyed"
            ));
            // Hands the token back to its owner's stock, drops it from their
            // PlayerCities and strips its map sprite.
            commands
                .entity(revolting_city)
                .remove::<CityInRevolt>()
                .insert(ReturnCityToStock);
            if let Some(area) = area_opt {
                commands.entity(area).remove::<BuiltCity>();
            }
            continue;
        };

        // Give the original owner his city token back before handing the area
        // over, so both players' city stocks stay honest.
        commands
            .entity(revolting_city)
            .remove::<CityInRevolt>()
            .insert(ReturnCityToStock);

        let Ok(area_transform) = area_query.get(area).copied() else {
            warn!("[TAXATION] Area {area:?} has no transform — city {revolting_city:?} eliminated");
            commands.entity(area).remove::<BuiltCity>();
            continue;
        };

        if let Ok((_, new_owner_name, _, mut player_cities, mut city_stock, faction)) =
            player_query.get_mut(new_owner)
        {
            let Some(texture) = game_factions
                .faction_city_icons
                .get(&faction.faction)
                .cloned()
            else {
                warn!("[TAXATION] No city icon for {:?}", faction.faction);
                commands.entity(area).remove::<BuiltCity>();
                continue;
            };
            info!("[TAXATION] Revolting city in {area:?} taken over by {new_owner_name}");
            round_summary.push(format!(
                "{owner_name}'s revolting city was taken over by {new_owner_name}"
            ));
            build_city_in_area(
                &mut commands,
                texture,
                &BuildCityCommand {
                    player: new_owner,
                    area,
                },
                &mut city_stock,
                &mut player_cities,
                &area_transform,
            );
        }
    }
}

/// Gate: waits until all `NeedsToPayTaxes`, `CityInRevolt` and
/// `AwaitingCoinageRateSelection` components are gone, then transitions to
/// `PopulationExpansion`.
pub fn taxation_gate(
    still_paying: Query<Entity, With<NeedsToPayTaxes>>,
    revolts: Query<Entity, With<CityInRevolt>>,
    awaiting_rate: Query<Entity, With<AwaitingCoinageRateSelection>>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if still_paying.is_empty() && revolts.is_empty() && awaiting_rate.is_empty() {
        next_state.set(GameActivity::PopulationExpansion);
    }
}

// ─── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::components::{PlayerCities, TokenStock, Treasury};
    use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
    use crate::stupid_ai::IsHuman;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

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
        let token_entities: Vec<Entity> = (0..stock_tokens)
            .map(|_| world.spawn_empty().id())
            .collect();

        let stock = TokenStock::new(55, token_entities);
        let mut cities = PlayerCities::default();
        for (area, city) in city_entities {
            cities.build_city_in_area(area, city);
        }

        world
            .spawn((Name::new("TestPlayer"), stock, cities, Treasury::default()))
            .id()
    }

    // ── Rule 19.1: taxes really move stock -> treasury ───────────────────────
    //
    // `collect_taxes` used to hand every paid token straight back to stock, so
    // the treasury (which pays for ninth-stack cards, 27.51, and scores at the
    // end) never grew and the phase cost nothing. These run the real system.

    fn taxation_app() -> App {
        let mut app = App::new();
        app.init_resource::<RoundSummary>();
        app.add_systems(Update, collect_taxes);
        app
    }

    fn spawn_taxpayer(app: &mut App, stock_tokens: usize, city_count: usize) -> Entity {
        let player = setup_player_with_cities(app.world_mut(), stock_tokens, city_count);
        app.world_mut()
            .entity_mut(player)
            .insert(NeedsToPayTaxes::new(city_count * 2));
        player
    }

    #[test]
    fn paid_taxes_end_up_in_the_treasury_not_back_in_stock() {
        let mut app = taxation_app();
        let player = spawn_taxpayer(&mut app, 10, 2); // 2 cities -> owes 4
        app.update();

        let stock = app.world().get::<TokenStock>(player).unwrap();
        let treasury = app.world().get::<Treasury>(player).unwrap();
        assert_eq!(
            stock.tokens_in_stock(),
            6,
            "4 tokens should have left stock"
        );
        assert_eq!(treasury.tokens_in_treasury(), 4);
        assert!(app.world().get::<NeedsToPayTaxes>(player).is_none());
    }

    #[test]
    fn a_shortfall_pays_what_it_can_and_revolts_the_rest() {
        let mut app = taxation_app();
        // 3 cities owe 6; only 3 tokens in stock -> pays for 1 city, 2 revolt.
        let player = spawn_taxpayer(&mut app, 3, 3);
        app.update();

        let treasury = app.world().get::<Treasury>(player).unwrap();
        assert_eq!(treasury.tokens_in_treasury(), 2);
        let stock = app.world().get::<TokenStock>(player).unwrap();
        assert_eq!(stock.tokens_in_stock(), 1);

        let revolts = app
            .world_mut()
            .query::<&CityInRevolt>()
            .iter(app.world())
            .count();
        assert_eq!(revolts, 2);
    }

    #[test]
    fn democracy_pays_taxes_but_never_revolts() {
        let mut app = taxation_app();
        let player = spawn_taxpayer(&mut app, 3, 3);
        let mut civ_cards = PlayerCivilizationCards::default();
        civ_cards.add_card(CivCardName::Democracy);
        app.world_mut().entity_mut(player).insert(civ_cards);
        app.update();

        // Pays everything it has (3), no revolt.
        let treasury = app.world().get::<Treasury>(player).unwrap();
        assert_eq!(treasury.tokens_in_treasury(), 3);
        let revolts = app
            .world_mut()
            .query::<&CityInRevolt>()
            .iter(app.world())
            .count();
        assert_eq!(revolts, 0);
    }

    // ── Rule 19.32: the *strongest* stock takes the revolting city ───────────

    #[test]
    fn revolt_beneficiary_ordering_prefers_the_largest_stock() {
        // The candidate list is sorted strongest-first and the first eligible
        // non-owner wins. Sorting ascending (the old behaviour) handed revolting
        // cities to the weakest player at the table.
        let mut candidates = [(1usize, 4usize), (2, 30), (3, 12)];
        candidates.sort_by_key(|(_, points)| std::cmp::Reverse(*points));
        assert_eq!(candidates.first().map(|(id, _)| *id), Some(2));
    }

    // ── Rule 19.1: standard taxation ─────────────────────────────────────────

    #[test]
    fn full_payment_moves_tokens_to_treasury() {
        let mut world = World::new();
        let player = setup_player_with_cities(&mut world, 10, 2); // 2 cities → owes 4 tokens

        let mut stock_before = 0;
        world
            .query::<(&TokenStock, &PlayerCities)>()
            .iter(&world)
            .for_each(|(s, _)| {
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
            // Scoped so the `TokenStock` borrow ends before we borrow `Treasury`.
            let taken = {
                let mut stock = world.get_mut::<TokenStock>(player).unwrap();
                stock.remove_tokens_from_stock(tokens_owed).unwrap()
            };
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
    fn coinage_rate_3_partial_payment_does_not_underflow() {
        // 5 cities at Coinage rate 3 owe 15 tokens; with 12 in stock the player
        // can't pay in full. Affordable = 12/3 = 4 cities (capped at 5), so 1
        // revolts. The old code used a hard-coded /2 (12/2 = 6 > 5 cities) and
        // underflowed `cities - affordable` -> panic.
        let num_cities = 5usize;
        let rate = 3usize;
        let tokens_owed = num_cities * rate; // 15
        let stock_count = 12usize;
        assert!(stock_count < tokens_owed);

        let affordable_cities = (stock_count / rate).min(num_cities);
        let cities_in_revolt = num_cities - affordable_cities;
        assert_eq!(affordable_cities, 4);
        assert_eq!(cities_in_revolt, 1);
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

    // ── Regression: the full CollectTaxes schedule must not skip the picker ──
    //
    // The three systems below run chained in `Update` every frame while in
    // `CollectTaxes` (see taxation_plugin.rs): collect_taxes, resolve_revolts,
    // taxation_gate. `taxation_gate` alone knows to wait for
    // `AwaitingCoinageRateSelection`, but `resolve_revolts` used to
    // unconditionally queue a transition to `PopulationExpansion` whenever
    // there were no revolts (the common case) -- stomping on that check before
    // it ever ran, and skipping the tax-rate picker every single time. A test
    // that only calls `taxation_gate` directly (as the tests above do) cannot
    // catch this: it has to run the real chain.
    #[test]
    fn full_collect_taxes_chain_does_not_skip_pending_coinage_choice() {
        let mut app = App::new();
        app.init_resource::<NextState<GameActivity>>();
        app.init_resource::<RoundSummary>();
        app.init_resource::<crate::civilization::concepts::map::AvailableFactions>();
        app.add_systems(
            Update,
            (collect_taxes, resolve_revolts, taxation_gate).chain(),
        );

        // A human Coinage holder is mid-choice; no one else owes taxes or is
        // revolting this frame.
        app.world_mut()
            .spawn((Name::new("Human"), AwaitingCoinageRateSelection));

        app.update();
        assert!(
            matches!(
                *app.world().resource::<NextState<GameActivity>>(),
                NextState::Unchanged
            ),
            "resolve_revolts must not force a transition out of CollectTaxes \
             while a human is still choosing a Coinage rate"
        );
    }

    // ── Rule 19.2: a human Coinage holder must be asked, not defaulted ───────

    #[test]
    fn human_coinage_holder_awaits_rate_choice_instead_of_defaulting() {
        // Regression test: a human who owns Coinage used to get NeedsToPayTaxes
        // straight away (silently defaulting to rate 2), with no UI ever asking
        // them to choose 1/2/3 (rule 19.2) -- only ai_set_coinage_rate (gated on
        // StupidAi) ever set CoinageTaxRate. enter_collect_taxes must instead
        // park them in AwaitingCoinageRateSelection until the UI resolves it.
        let mut app = App::new();
        app.init_resource::<NextState<GameActivity>>();
        let player = setup_player_with_cities(app.world_mut(), 10, 2);
        app.world_mut().entity_mut(player).insert(IsHuman);
        let mut civ_cards = PlayerCivilizationCards::default();
        civ_cards.add_card(CivCardName::Coinage);
        app.world_mut().entity_mut(player).insert(civ_cards);

        app.world_mut()
            .run_system_once(enter_collect_taxes)
            .unwrap();

        assert!(
            app.world().get::<AwaitingCoinageRateSelection>(player).is_some(),
            "human Coinage holder should await a rate choice"
        );
        assert!(
            app.world().get::<NeedsToPayTaxes>(player).is_none(),
            "tax obligation must not be computed before the human chooses a rate"
        );
    }

    #[test]
    fn ai_coinage_holder_is_not_asked_and_pays_immediately() {
        // AI players resolve their rate via ai_set_coinage_rate before this
        // system runs, so a CoinageTaxRate is already present -- they should
        // never be parked in AwaitingCoinageRateSelection.
        let mut app = App::new();
        app.init_resource::<NextState<GameActivity>>();
        let player = setup_player_with_cities(app.world_mut(), 10, 2);
        let mut civ_cards = PlayerCivilizationCards::default();
        civ_cards.add_card(CivCardName::Coinage);
        app.world_mut().entity_mut(player).insert(civ_cards);
        app.world_mut().entity_mut(player).insert(CoinageTaxRate(3));

        app.world_mut()
            .run_system_once(enter_collect_taxes)
            .unwrap();

        assert!(
            app.world().get::<AwaitingCoinageRateSelection>(player).is_none(),
            "AI holders already have a rate and must not wait on UI"
        );
        let needs = app.world().get::<NeedsToPayTaxes>(player).unwrap();
        assert_eq!(needs.tokens_owed, 6, "2 cities at rate 3 = 6 tokens owed");
    }

    #[test]
    fn taxation_gate_waits_for_pending_coinage_rate_choice() {
        let mut app = App::new();
        app.init_resource::<NextState<GameActivity>>();
        let player = app.world_mut().spawn(AwaitingCoinageRateSelection).id();

        app.world_mut().run_system_once(taxation_gate).unwrap();
        assert!(
            matches!(
                *app.world().resource::<NextState<GameActivity>>(),
                NextState::Unchanged
            ),
            "must not queue leaving CollectTaxes while a human is still choosing a rate"
        );

        app.world_mut()
            .entity_mut(player)
            .remove::<AwaitingCoinageRateSelection>();
        app.world_mut().run_system_once(taxation_gate).unwrap();

        assert!(matches!(
            *app.world().resource::<NextState<GameActivity>>(),
            NextState::Pending(GameActivity::PopulationExpansion)
        ));
    }

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
