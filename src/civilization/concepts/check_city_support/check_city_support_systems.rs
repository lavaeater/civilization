use crate::civilization::components::*;
use crate::civilization::concepts::check_city_support::check_city_support_components::*;
use crate::civilization::concepts::check_city_support::check_city_support_events::*;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::GameActivity;
use bevy::prelude::{info, Commands, Entity, MessageReader, MessageWriter, NextState, Query, ResMut, With};

pub fn eliminate_city(
    mut eliminate_city: MessageReader<EliminateCity>,
    mut commands: Commands,
    mut city_token_stock: Query<(&mut CityTokenStock, &mut PlayerCities)>,
    area_population: Query<&mut Population>,
    city_token: Query<&CityToken>,
    mut move_tokens: MessageWriter<MoveTokensFromStockToAreaCommand>,
) {
    for eliminate in eliminate_city.read() {
        //Remove TooManyCities
        commands
            .entity(eliminate.player)
            .remove::<HasTooManyCities>();
        if let Ok(city_token) = city_token.get(eliminate.city) {
            if let Ok((mut city_stock, mut player_cities)) =
                city_token_stock.get_mut(city_token.player)
            {
                if let Ok(population) = area_population.get(eliminate.area_entity) {
                    //debug!("Eliminating city, conflict: {}, max_pop: {}", eliminate.is_conflict, population.max_population);
                    move_tokens.write(MoveTokensFromStockToAreaCommand {
                        player_entity: city_token.player,
                        area_entity: eliminate.area_entity,
                        number_of_tokens: if eliminate.is_conflict {
                            6
                        } else {
                            population.max_population
                        },
                    });
                    commands.entity(eliminate.area_entity).remove::<BuiltCity>();
                    player_cities.remove_city_from_area(eliminate.area_entity);
                    city_stock.return_token_to_stock(eliminate.city);
                }
            }
        }
        commands
            .entity(eliminate.player)
            .insert(NeedsToCheckCitySupport); //Start check all over again to update too many cities thingie!
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

        if required_population > areas.total_population() {
            //debug!("A player has too many cities");
            commands.entity(player).insert(HasTooManyCities::new(
                (required_population - areas.total_population()) / 2,
                required_population - areas.total_population(),
            ));
        } else {
            commands.entity(player).remove::<HasTooManyCities>();
        }
        commands.entity(player).remove::<NeedsToCheckCitySupport>();
    }
}

pub fn start_check_city_support(
    player_cities_query: Query<(Entity, &PlayerCities)>,
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
