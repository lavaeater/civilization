use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::rng;

use crate::civilization::components::*;
use crate::civilization::{PlayerTradeCards, TradeCard, TradeCardTrait};
use crate::civilization::concepts::resolve_calamities::resolve_calamities_components::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_events::*;
use crate::civilization::functions::return_all_tokens_to_stock;
use crate::player::Player;
use crate::GameActivity;

pub fn start_calamity_resolution(
    mut commands: Commands,
    players_with_calamities: Query<(Entity, &PlayerTradeCards), With<Player>>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    info!("[CALAMITIES] Starting calamity resolution phase");
    
    let mut any_calamities = false;
    
    for (player_entity, trade_cards) in players_with_calamities.iter() {
        let calamity_cards: Vec<TradeCard> = trade_cards.calamity_cards().iter().cloned().collect();
        
        if calamity_cards.is_empty() {
            continue;
        }
        
        any_calamities = true;
        info!("[CALAMITIES] Player {:?} has {} calamities", player_entity, calamity_cards.len());
        
        let calamities_to_resolve = if calamity_cards.len() > 2 {
            select_random_calamities(&calamity_cards, 2)
        } else {
            calamity_cards.iter().map(|c| (*c, None)).collect()
        };
        
        commands.entity(player_entity).insert(PendingCalamities::new(calamities_to_resolve));
        commands.entity(player_entity).insert(NeedsCalamityResolution);
    }
    
    if !any_calamities {
        info!("[CALAMITIES] No calamities to resolve, skipping phase");
        next_state.set(GameActivity::CheckCitySupport);
    }
}

fn select_random_calamities(calamities: &[TradeCard], count: usize) -> Vec<(TradeCard, Option<Entity>)> {
    let mut rng = rng();
    let mut shuffled: Vec<TradeCard> = calamities.to_vec();
    shuffled.shuffle(&mut rng);
    shuffled.into_iter().take(count).map(|c| (c, None)).collect()
}

pub fn process_pending_calamities(
    mut commands: Commands,
    mut players_with_pending: Query<(Entity, &mut PendingCalamities, &mut PlayerTradeCards), With<NeedsCalamityResolution>>,
    mut resolve_volcano_earthquake: MessageWriter<ResolveVolcanoEarthquake>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    let mut all_calamities: Vec<(Entity, TradeCard, Option<Entity>)> = Vec::new();
    
    for (player_entity, pending, _) in players_with_pending.iter() {
        for (calamity, traded_by) in pending.calamities.iter() {
            all_calamities.push((player_entity, *calamity, *traded_by));
        }
    }
    
    if all_calamities.is_empty() {
        for (player_entity, _, _) in players_with_pending.iter() {
            commands.entity(player_entity).remove::<NeedsCalamityResolution>();
            commands.entity(player_entity).remove::<PendingCalamities>();
        }
        info!("[CALAMITIES] All calamities resolved, transitioning to CheckCitySupport");
        next_state.set(GameActivity::CheckCitySupport);
        return;
    }
    
    all_calamities.sort_by(|a, b| {
        let value_cmp = a.1.value().cmp(&b.1.value());
        if value_cmp != std::cmp::Ordering::Equal {
            return value_cmp;
        }
        let a_tradeable = a.1.is_tradeable();
        let b_tradeable = b.1.is_tradeable();
        a_tradeable.cmp(&b_tradeable)
    });
    
    if let Some((player_entity, calamity, traded_by)) = all_calamities.first() {
        info!("[CALAMITIES] Resolving {:?} for player {:?}", calamity, player_entity);
        
        if let Ok((_, mut pending, mut trade_cards)) = players_with_pending.get_mut(*player_entity) {
            pending.calamities.retain(|(c, _)| c != calamity);
            let _ = trade_cards.remove_n_trade_cards(1, *calamity);
            
            match calamity {
                TradeCard::VolcanoEarthquake => {
                    resolve_volcano_earthquake.write(ResolveVolcanoEarthquake {
                        primary_victim: *player_entity,
                        traded_by: *traded_by,
                    });
                }
                _ => {
                    info!("[CALAMITIES] Calamity {:?} not yet implemented", calamity);
                }
            }
        }
    }
}

pub fn resolve_volcano_earthquake(
    mut events: MessageReader<ResolveVolcanoEarthquake>,
    mut commands: Commands,
    player_cities: Query<&PlayerCities>,
    area_query: Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
    volcano_areas: Query<Entity, With<Volcano>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    names: Query<&Name>,
) {
    for event in events.read() {
        let primary_victim = event.primary_victim;
        let player_name = names.get(primary_victim).map(|n| n.to_string()).unwrap_or_else(|_| "Unknown".to_string());
        
        info!("[VOLCANO/EARTHQUAKE] Resolving for {}", player_name);
        
        let player_cities_component = player_cities.get(primary_victim).ok();
        
        let mut volcano_candidates: Vec<(Entity, usize, bool)> = Vec::new();
        
        for volcano_area in volcano_areas.iter() {
            if let Ok((_area_entity, population, built_city, _, land_passage)) = area_query.get(volcano_area) {
                let mut total_damage = 0usize;
                let mut victim_has_city_in_touched_areas = false;
                
                if let Some(city) = built_city {
                    if city.player == primary_victim {
                        total_damage += 5;
                        victim_has_city_in_touched_areas = true;
                    }
                }
                
                if let Some(tokens) = population.tokens_for_player(&primary_victim) {
                    total_damage += tokens.len();
                }
                
                for adjacent_area in land_passage.to_areas.iter() {
                    if let Ok((_, adj_pop, adj_city, _, _)) = area_query.get(*adjacent_area) {
                        if let Some(city) = adj_city {
                            if city.player == primary_victim {
                                total_damage += 5;
                                victim_has_city_in_touched_areas = true;
                            }
                        }
                        if let Some(tokens) = adj_pop.tokens_for_player(&primary_victim) {
                            total_damage += tokens.len();
                        }
                    }
                }
                
                if victim_has_city_in_touched_areas {
                    volcano_candidates.push((volcano_area, total_damage, true));
                }
            }
        }
        
        if !volcano_candidates.is_empty() {
            volcano_candidates.sort_by_key(|b| std::cmp::Reverse(b.1));
            let (volcano_area, _, _) = volcano_candidates[0];
            
            info!("[VOLCANO] Eruption at area {:?}", volcano_area);
            
            trigger_volcano_eruption(&mut commands, volcano_area, &area_query);
        } else {
            info!("[EARTHQUAKE] No volcano areas with cities, triggering earthquake");
            
            if let Some(cities) = player_cities_component {
                let city_areas: Vec<Entity> = cities.areas_and_cities.keys().cloned().collect();
                
                if !city_areas.is_empty() {
                    let city_to_destroy = city_areas[0];
                    
                    let mut adjacent_cities: Vec<Entity> = Vec::new();
                    if let Ok((_, _, _, _, land_passage)) = area_query.get(city_to_destroy) {
                        for adjacent_area in land_passage.to_areas.iter() {
                            if let Ok((_, _, Some(adj_city), _, _)) = area_query.get(*adjacent_area) {
                                if adj_city.player != primary_victim {
                                    adjacent_cities.push(*adjacent_area);
                                }
                            }
                        }
                    }
                    
                    let city_to_reduce = adjacent_cities.first().cloned();
                    
                    info!("[EARTHQUAKE] Destroying city at {:?}, reducing city at {:?}", city_to_destroy, city_to_reduce);
                    
                    trigger_earthquake(&mut commands, primary_victim, city_to_destroy, city_to_reduce, &area_query);
                }
            }
        }
        
        calamity_resolved.write(CalamityResolved {
            player: primary_victim,
            calamity: TradeCard::VolcanoEarthquake,
        });
    }
}

fn trigger_volcano_eruption(
    commands: &mut Commands,
    volcano_area: Entity,
    area_query: &Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
) {
    let mut areas_to_clear: Vec<Entity> = vec![volcano_area];
    
    if let Ok((_, _, _, _, land_passage)) = area_query.get(volcano_area) {
        areas_to_clear.extend(land_passage.to_areas.iter().cloned());
    }
    
    for area in areas_to_clear.iter() {
        if let Ok((area_entity, _population, built_city, _, _)) = area_query.get(*area) {
            commands.entity(area_entity).insert(FixTokenPositions);
            
            if let Some(city) = built_city {
                commands.entity(area_entity).remove::<BuiltCity>();
                commands.entity(city.city).insert(ReturnCityToStock);
            }
        }
    }
    
    for area in areas_to_clear.iter() {
        commands.entity(*area).insert(ClearAllTokens);
    }
}

fn trigger_earthquake(
    commands: &mut Commands,
    _primary_victim: Entity,
    city_to_destroy: Entity,
    city_to_reduce: Option<Entity>,
    _area_query: &Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
) {
    commands.entity(city_to_destroy).insert(DestroyCity);
    
    if let Some(reduce_area) = city_to_reduce {
        commands.entity(reduce_area).insert(ReduceCity);
    }
}

pub fn handle_calamity_resolved(
    mut events: MessageReader<CalamityResolved>,
    players_with_pending: Query<(Entity, &PendingCalamities), With<NeedsCalamityResolution>>,
    mut commands: Commands,
    names: Query<&Name>,
) {
    for event in events.read() {
        let player_name = names.get(event.player).map(|n| n.to_string()).unwrap_or_else(|_| "Unknown".to_string());
        info!("[CALAMITIES] {} resolved {:?}", player_name, event.calamity);
        
        if let Ok((player_entity, pending)) = players_with_pending.get(event.player) {
            if pending.is_empty() {
                commands.entity(player_entity).remove::<NeedsCalamityResolution>();
                commands.entity(player_entity).remove::<PendingCalamities>();
            }
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ClearAllTokens;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DestroyCity;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ReduceCity;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ReturnCityToStock;

pub fn clear_all_tokens_from_area(
    mut commands: Commands,
    mut areas_to_clear: Query<(Entity, &mut Population), With<ClearAllTokens>>,
) {
    for (area_entity, mut population) in areas_to_clear.iter_mut() {
        info!("[CALAMITIES] Clearing all tokens from area {:?}", area_entity);
        return_all_tokens_to_stock(&mut population, &mut commands);
        commands.entity(area_entity).remove::<ClearAllTokens>();
        commands.entity(area_entity).insert(FixTokenPositions);
    }
}

pub fn destroy_city_in_area(
    mut commands: Commands,
    areas_with_destroy: Query<(Entity, &BuiltCity), With<DestroyCity>>,
    mut city_stock_query: Query<(&mut CityTokenStock, &mut PlayerCities)>,
) {
    for (area_entity, built_city) in areas_with_destroy.iter() {
        info!("[CALAMITIES] Destroying city in area {:?}", area_entity);
        
        if let Ok((mut city_stock, mut player_cities)) = city_stock_query.get_mut(built_city.player) {
            player_cities.remove_city_from_area(area_entity);
            city_stock.return_token_to_stock(built_city.city);
        }
        
        commands.entity(area_entity).remove::<BuiltCity>();
        commands.entity(area_entity).remove::<DestroyCity>();
    }
}

pub fn reduce_city_in_area(
    mut commands: Commands,
    areas_with_reduce: Query<(Entity, &BuiltCity, &Population), With<ReduceCity>>,
    mut city_stock_query: Query<(&mut CityTokenStock, &mut PlayerCities, &mut TokenStock, &mut PlayerAreas)>,
    mut move_tokens: MessageWriter<crate::civilization::events::MoveTokensFromStockToAreaCommand>,
) {
    for (area_entity, built_city, population) in areas_with_reduce.iter() {
        info!("[CALAMITIES] Reducing city in area {:?}", area_entity);
        
        if let Ok((mut city_stock, mut player_cities, _, _)) = city_stock_query.get_mut(built_city.player) {
            player_cities.remove_city_from_area(area_entity);
            city_stock.return_token_to_stock(built_city.city);
            
            move_tokens.write(crate::civilization::events::MoveTokensFromStockToAreaCommand {
                player_entity: built_city.player,
                area_entity,
                number_of_tokens: population.max_population.min(6),
            });
        }
        
        commands.entity(area_entity).remove::<BuiltCity>();
        commands.entity(area_entity).remove::<ReduceCity>();
    }
}
