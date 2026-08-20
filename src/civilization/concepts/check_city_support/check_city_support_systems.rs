use crate::civilization::components::*;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::CivCardName;
use crate::civilization::concepts::check_city_support::check_city_support_components::*;
use crate::civilization::concepts::check_city_support::check_city_support_events::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_components::PirateNation;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::triggers::retire_city_token_visuals;
use crate::GameActivity;
use bevy::prelude::{error, info, Commands, Entity, MessageReader, MessageWriter, NextState, Query, ResMut, With, Without};

pub fn eliminate_city(
    mut eliminate_city: MessageReader<EliminateCity>,
    mut commands: Commands,
    mut city_token_stock: Query<(&mut CityTokenStock, &mut PlayerCities)>,
    area_population: Query<&mut Population>,
    city_token: Query<&CityToken>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut move_tokens: MessageWriter<MoveTokensFromStockToAreaCommand>,
) {
    for eliminate in eliminate_city.read() {
        //Remove TooManyCities
        commands
            .entity(eliminate.player)
            .remove::<HasTooManyCities>();

        if let Ok(city_token) = city_token.get(eliminate.city)
            && let Ok((mut city_stock, mut player_cities)) =
                city_token_stock.get_mut(city_token.player)
            && let Ok(population) = area_population.get(eliminate.area_entity)
        {
            //debug!("Eliminating city, conflict: {}, max_pop: {}", eliminate.is_conflict, population.max_population);
            // Rule 26.11/26.41: a city reduced (not conflict-eliminated) is
            // replaced by tokens up to the area's population limit; that limit
            // is +1 if the reducing player holds Agriculture, since the area
            // is necessarily solely occupied by their own replacement tokens
            // afterward (26.1 -- areas with cities may not also contain
            // tokens, so there's nothing else there to disqualify the bonus).
            let has_agriculture = civ_cards_query
                .get(city_token.player)
                .is_ok_and(|c| c.owns(&CivCardName::Agriculture));
            move_tokens.write(MoveTokensFromStockToAreaCommand {
                player_entity: city_token.player,
                area_entity: eliminate.area_entity,
                number_of_tokens: if eliminate.is_conflict {
                    6
                } else {
                    population.max_population + usize::from(has_agriculture)
                },
            });
            commands.entity(eliminate.area_entity).remove::<BuiltCity>();
            let removed = player_cities.remove_city_from_area(eliminate.area_entity);
            city_stock.return_token_to_stock(eliminate.city);
            retire_city_token_visuals(&mut commands, eliminate.city);

            if removed.is_some() {
                // Only re-check once the city is actually gone. Doing this
                // unconditionally would spin forever if the removal is a no-op
                // (e.g. stale ownership): the player gets re-flagged, regenerates
                // the same still-present city move, and lands right back here.
                commands
                    .entity(eliminate.player)
                    .insert(NeedsToCheckCitySupport); //Start check all over again to update too many cities thingie!
            } else {
                error!(
                    "[CITY_SUPPORT] eliminate_city resolved the city token but \
                     area {:?} was not in {:?}'s cities (stale ownership?); \
                     not re-checking to avoid an infinite loop.",
                    eliminate.area_entity, city_token.player
                );
            }
        } else {
            // The move pointed at a city/area/stock we can't resolve. Don't
            // re-flag the player (that would loop the whole phase forever);
            // log it loudly so the stale move source can be found and fixed.
            error!(
                "[CITY_SUPPORT] eliminate_city could not resolve move \
                 (player={:?}, city={:?}, area={:?}); dropping it without \
                 re-checking to avoid an infinite loop.",
                eliminate.player, eliminate.city, eliminate.area_entity
            );
        }
    }
}

pub fn check_status_after_remove_surplus_population(
    needs_city_support: Query<&HasTooManyCities>,
    needs_to_check_city_support: Query<&NeedsToCheckCitySupport>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let too_many_cities_count = needs_city_support.iter().count();
    let needs_check_count = needs_to_check_city_support.iter().count();
    
    if too_many_cities_count == 0 && needs_check_count == 0 {
        info!("[CITY_SUPPORT] All checks complete, transitioning to AcquireTradeCards");
        next_state.set(GameActivity::AcquireTradeCards);
    }
}

pub fn check_status_after_resolve_calamities(
    needs_city_support: Query<&HasTooManyCities>,
    needs_to_check_city_support: Query<&NeedsToCheckCitySupport>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let too_many_cities_count = needs_city_support.iter().count();
    let needs_check_count = needs_to_check_city_support.iter().count();

    if too_many_cities_count == 0 && needs_check_count == 0 {
        info!("[CITY_SUPPORT] All checks complete, transitioning to AcquireCivilizationCards");
        next_state.set(GameActivity::AcquireCivilizationCards);
    }
}

pub fn check_player_city_support(
    check_city_support_query: Query<
        (Entity, &PlayerCities, &PlayerAreas),
        With<NeedsToCheckCitySupport>,
    >,
    mut commands: Commands,
) {
    for (player, cities, areas) in check_city_support_query.iter() {
        let number_of_cities = cities.number_of_cities();
        let required_population = number_of_cities * 2;
        let total_population = areas.total_population();

        if required_population > total_population {
            info!(
                "[CITY_SUPPORT] {:?} has too many cities: {} cities need {} population, has {}",
                player, number_of_cities, required_population, total_population
            );
            commands.entity(player).insert(HasTooManyCities::new(
                (required_population - total_population) / 2,
                required_population - total_population,
            ));
        } else {
            commands.entity(player).remove::<HasTooManyCities>();
        }
        commands.entity(player).remove::<NeedsToCheckCitySupport>();
    }
}

pub fn start_check_city_support(
    // Rule 30.913: Pirate cities never require city support. `PlayerCities`
    // isn't otherwise gated by `With<Player>` here (a bare-owner query, used
    // so eliminated/inactive players still get swept), so Piracy's shared
    // `PirateNation` owner entity -- which does hold `PlayerCities` for
    // `TransferCityTo` to work -- must be explicitly excluded or it would
    // get incorrectly flagged as under-supported (it has no `PlayerAreas`
    // population to support cities with, since Pirates never place tokens
    // there directly).
    player_cities_query: Query<(Entity, &PlayerCities), Without<PirateNation>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    info!("[CITY_SUPPORT] Starting city support check phase");
    
    if player_cities_query.is_empty()
        || player_cities_query
            .iter()
            .all(|(_, player_cities)| player_cities.has_no_cities())
    {
        info!("[CITY_SUPPORT] No players with cities, skipping to AcquireTradeCards");
        next_state.set(GameActivity::AcquireTradeCards);
    } else {
        let mut players_with_cities = 0;
        for (entity, player_cities) in player_cities_query.iter() {
            if player_cities.has_cities() {
                players_with_cities += 1;
                commands.entity(entity).insert(NeedsToCheckCitySupport {});
            }
        }
        info!("[CITY_SUPPORT] {} players need city support check", players_with_cities);
    }
}
