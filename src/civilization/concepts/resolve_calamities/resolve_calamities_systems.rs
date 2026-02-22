use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::rng;

use crate::civilization::components::*;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::resolve_calamities::calamities::volcano_earthquake::{VolcanoEarthquakePhase, VolcanoEarthquakeState};
use crate::civilization::concepts::resolve_calamities::calamities::ResolvingCalamity;
use crate::civilization::concepts::resolve_calamities::context::{ActiveCalamityResolution, CalamityContext, CalamityPhase};
use crate::civilization::{CivCardName, PlayerTradeCards, TradeCard, TradeCardTrait};
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
        next_state.set(GameActivity::CheckCitySupportAfterResolveCalamities);
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
        next_state.set(GameActivity::CheckCitySupportAfterResolveCalamities);
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
    player_civ_cards: Query<&PlayerCivilizationCards>,
    area_query: Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
    volcano_areas: Query<Entity, With<Volcano>>,
    names: Query<&Name>,
) {
    for event in events.read(){
        let primary_victim = event.primary_victim;
        let player_name = names.get(primary_victim).map(|n| n.to_string()).unwrap_or_else(|_| "Unknown".to_string());
        
        info!("[VOLCANO/EARTHQUAKE] Resolving for {}", player_name);
        
        let has_engineering = player_civ_cards
            .get(primary_victim)
            .map(|cards: &PlayerCivilizationCards| cards.owns(&CivCardName::Engineering))
            .unwrap_or(false);
        
        let player_cities_component = player_cities.get(primary_victim).ok();
        
        let volcano_result = find_best_volcano_eruption(
            primary_victim,
            &volcano_areas,
            &area_query,
        );
        
        let state = if let Some((volcano_area, areas_to_clear)) = volcano_result {
            info!("[VOLCANO] Eruption at area {:?}", volcano_area);
            VolcanoEarthquakeState::as_volcano(volcano_area, areas_to_clear)
        } else {
            info!("[EARTHQUAKE] No volcano areas with cities, triggering earthquake");
            
            if let Some(cities) = player_cities_component {
                let city_areas: Vec<Entity> = cities.areas_and_cities.keys().cloned().collect();
                
                if !city_areas.is_empty() {
                    let city_to_destroy = city_areas[0];
                    let city_to_reduce = find_adjacent_city_to_reduce(
                        city_to_destroy,
                        primary_victim,
                        &area_query,
                    );
                    
                    info!("[EARTHQUAKE] City to destroy: {:?}, city to reduce: {:?}, has_engineering: {}", 
                        city_to_destroy, city_to_reduce, has_engineering);
                    
                    VolcanoEarthquakeState::as_earthquake(city_to_destroy, city_to_reduce, has_engineering)
                } else {
                    VolcanoEarthquakeState::new()
                }
            } else {
                VolcanoEarthquakeState::new()
            }
        };
        
        let context = CalamityContext::new(
            TradeCard::VolcanoEarthquake,
            primary_victim,
            event.traded_by,
        );
        
        commands.entity(primary_victim).insert((
            ActiveCalamityResolution::new(context),
            ResolvingCalamity::VolcanoEarthquake(state),
        ));
    }
}

fn find_best_volcano_eruption(
    primary_victim: Entity,
    volcano_areas: &Query<Entity, With<Volcano>>,
    area_query: &Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
) -> Option<(Entity, Vec<Entity>)> {
    let mut volcano_candidates: Vec<(Entity, usize, Vec<Entity>)> = Vec::new();
    
    for volcano_area in volcano_areas.iter() {
        if let Ok((_area_entity, population, built_city, _, land_passage)) = area_query.get(volcano_area) {
            let mut total_damage = 0usize;
            let mut victim_has_city_in_touched_areas = false;
            let mut areas_to_clear = vec![volcano_area];
            
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
                areas_to_clear.push(*adjacent_area);
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
                volcano_candidates.push((volcano_area, total_damage, areas_to_clear));
            }
        }
    }
    
    if volcano_candidates.is_empty() {
        return None;
    }
    
    volcano_candidates.sort_by_key(|b| std::cmp::Reverse(b.1));
    let (volcano_area, _, areas_to_clear) = volcano_candidates.remove(0);
    Some((volcano_area, areas_to_clear))
}

fn find_adjacent_city_to_reduce(
    city_area: Entity,
    primary_victim: Entity,
    area_query: &Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
) -> Option<Entity> {
    if let Ok((_, _, _, _, land_passage)) = area_query.get(city_area) {
        for adjacent_area in land_passage.to_areas.iter() {
            if let Ok((_, _, Some(adj_city), _, _)) = area_query.get(*adjacent_area) {
                if adj_city.player != primary_victim {
                    return Some(*adjacent_area);
                }
            }
        }
    }
    None
}

pub fn apply_volcano_earthquake_effects(
    mut commands: Commands,
    mut players_resolving: Query<(Entity, &mut ActiveCalamityResolution, &ResolvingCalamity)>,
    area_query: Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolution, resolving) in players_resolving.iter_mut() {
        if resolution.phase != CalamityPhase::ComputeEffects {
            continue;
        }
        
        if let ResolvingCalamity::VolcanoEarthquake(state) = resolving {
            if state.phase != VolcanoEarthquakePhase::ApplyEffects {
                continue;
            }
            
            if state.is_volcano {
                if let Some(volcano_area) = state.volcano_area {
                    trigger_volcano_eruption(&mut commands, volcano_area, &area_query);
                }
            } else {
                if let Some(city_to_destroy) = state.city_to_destroy {
                    commands.entity(city_to_destroy).insert(DestroyCity);
                }
                if let Some(city_to_reduce) = state.city_to_reduce {
                    commands.entity(city_to_reduce).insert(ReduceCity);
                }
            }
            
            resolution.mark_resolved();
            
            calamity_resolved.write(CalamityResolved {
                player: player_entity,
                calamity: TradeCard::VolcanoEarthquake,
            });
            
            commands.entity(player_entity).remove::<ResolvingCalamity>();
            commands.entity(player_entity).remove::<ActiveCalamityResolution>();
        }
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
