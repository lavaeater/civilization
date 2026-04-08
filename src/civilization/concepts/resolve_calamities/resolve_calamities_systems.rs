use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::rng;

use crate::civilization::components::*;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::resolve_calamities::calamities::barbarian_hordes::{BarbarianHordesPhase, BarbarianHordesState};
use crate::civilization::concepts::resolve_calamities::calamities::civil_disorder::{CivilDisorderPhase, CivilDisorderState};
use crate::civilization::concepts::resolve_calamities::calamities::civil_war::{CivilWarPhase, CivilWarState};
use crate::civilization::concepts::resolve_calamities::calamities::epidemic::{EpidemicPhase, EpidemicState};
use crate::civilization::concepts::resolve_calamities::calamities::famine::{FaminePhase, FamineState};
use crate::civilization::concepts::resolve_calamities::calamities::flood::{FloodPhase, FloodState};
use crate::civilization::concepts::resolve_calamities::calamities::iconoclasm_heresy::{IconoclasmHeresyPhase, IconoclasmHeresyState};
use crate::civilization::concepts::resolve_calamities::calamities::piracy::{PiracyPhase, PiracyState};
use crate::civilization::concepts::resolve_calamities::calamities::slave_revolt::{SlaveRevoltPhase, SlaveRevoltState};
use crate::civilization::concepts::resolve_calamities::calamities::superstition::{SuperstitionPhase, SuperstitionState};
use crate::civilization::concepts::resolve_calamities::calamities::treachery::{TreacheryPhase, TreacheryState};
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

/// Picks the next calamity to resolve (globally lowest value, non-tradeable first) and dispatches
/// it by attaching `ResolvingCalamity` + `ActiveCalamityResolution` to the victim entity.
/// Only runs when no calamity is currently being resolved.
pub fn process_pending_calamities(
    mut commands: Commands,
    mut players_with_pending: Query<(Entity, &mut PendingCalamities, &mut PlayerTradeCards), With<NeedsCalamityResolution>>,
    player_civ_cards: Query<&PlayerCivilizationCards>,
    existing_resolutions: Query<Entity, With<ResolvingCalamity>>,
    mut resolve_volcano_earthquake: MessageWriter<ResolveVolcanoEarthquake>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // One calamity at a time – wait until the current one finishes
    if !existing_resolutions.is_empty() {
        return;
    }

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

    // Resolve lowest-value calamities first; non-tradeable before tradeable at same value
    all_calamities.sort_by(|a, b| {
        let value_cmp = a.1.value().cmp(&b.1.value());
        if value_cmp != std::cmp::Ordering::Equal {
            return value_cmp;
        }
        a.1.is_tradeable().cmp(&b.1.is_tradeable())
    });

    if let Some((player_entity, calamity, traded_by)) = all_calamities.first() {
        info!("[CALAMITIES] Resolving {:?} for player {:?}", calamity, player_entity);

        let civ_cards = player_civ_cards.get(*player_entity).ok();

        if let Ok((_, mut pending, mut trade_cards)) = players_with_pending.get_mut(*player_entity) {
            pending.calamities.retain(|(c, _)| c != calamity);
            let _ = trade_cards.remove_n_trade_cards(1, *calamity);

            let context = CalamityContext::new(*calamity, *player_entity, *traded_by);

            match calamity {
                TradeCard::VolcanoEarthquake => {
                    // VolcanoEarthquake needs complex map queries, handled via message
                    resolve_volcano_earthquake.write(ResolveVolcanoEarthquake {
                        primary_victim: *player_entity,
                        traded_by: *traded_by,
                    });
                }
                TradeCard::Famine => {
                    let grain_count = trade_cards.number_of_cards_for_trade_card(TradeCard::Grain);
                    let has_pottery = civ_cards.map(|c| c.owns(&CivCardName::Pottery)).unwrap_or(false);
                    let state = FamineState::new().with_grain_reduction(grain_count, has_pottery);
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::Famine(state)));
                }
                TradeCard::Superstition => {
                    let state = if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Enlightenment) { SuperstitionState::new().with_enlightenment() }
                        else if c.owns(&CivCardName::Deism) { SuperstitionState::new().with_deism() }
                        else if c.owns(&CivCardName::Mysticism) { SuperstitionState::new().with_mysticism() }
                        else { SuperstitionState::new() }
                    } else { SuperstitionState::new() };
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::Superstition(state)));
                }
                TradeCard::CivilWar => {
                    let mut state = CivilWarState::new();
                    if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Music) { state.apply_music_bonus(); }
                        if c.owns(&CivCardName::DramaAndPoetry) { state.apply_drama_poetry_bonus(); }
                        if c.owns(&CivCardName::Democracy) { state.apply_democracy_bonus(); }
                    }
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::CivilWar(state)));
                }
                TradeCard::SlaveRevolt => {
                    // Default: reduce 2 cities; Enlightenment = 0
                    let has_enlightenment = civ_cards.map(|c| c.owns(&CivCardName::Enlightenment)).unwrap_or(false);
                    let state = if has_enlightenment { SlaveRevoltState::new(2).with_enlightenment() } else { SlaveRevoltState::new(2) };
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::SlaveRevolt(state)));
                }
                TradeCard::Flood => {
                    let has_engineering = civ_cards.map(|c| c.owns(&CivCardName::Engineering)).unwrap_or(false);
                    let mut state = FloodState::new();
                    if has_engineering { state = state.with_engineering(); }
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::Flood(state)));
                }
                TradeCard::BarbarianHordes => {
                    let has_military = civ_cards.map(|c| c.owns(&CivCardName::Military)).unwrap_or(false);
                    let state = if has_military { BarbarianHordesState::new().with_military() } else { BarbarianHordesState::new() };
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::BarbarianHordes(state)));
                }
                TradeCard::Epidemic => {
                    let has_medicine = civ_cards.map(|c| c.owns(&CivCardName::Medicine)).unwrap_or(false);
                    let mut state = if has_medicine { EpidemicState::new().with_medicine() } else { EpidemicState::new() };
                    if let Some(immune) = *traded_by { state = state.with_immune_player(immune); }
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::Epidemic(state)));
                }
                TradeCard::CivilDisorder => {
                    // Default: reduce 2 cities; Democracy = 0, Law = 1
                    let mut state = CivilDisorderState::new(2);
                    if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Democracy) { state = state.with_democracy(); }
                        else if c.owns(&CivCardName::Law) { state = state.with_law(); }
                    }
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::CivilDisorder(state)));
                }
                TradeCard::IconoclasmAndHeresy => {
                    let mut state = IconoclasmHeresyState::new();
                    if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Theology) { state = state.with_theology(); }
                        if c.owns(&CivCardName::Philosophy) { state = state.with_philosophy(); }
                    }
                    if let Some(immune) = *traded_by { state = state.with_immune_player(immune); }
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::IconoclasmAndHeresy(state)));
                }
                TradeCard::Treachery => {
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::Treachery(TreacheryState::new())));
                }
                TradeCard::Piracy => {
                    let mut state = PiracyState::new();
                    if let Some(immune) = *traded_by { state = state.with_immune_player(immune); }
                    commands.entity(*player_entity).insert((ActiveCalamityResolution::new(context), ResolvingCalamity::Piracy(state)));
                }
                _ => {
                    info!("[CALAMITIES] Unknown calamity {:?}", calamity);
                }
            }
        }
    }
}

// ── VolcanoEarthquake (existing implementation, unchanged) ────────────────────

pub fn resolve_volcano_earthquake(
    mut events: MessageReader<ResolveVolcanoEarthquake>,
    mut commands: Commands,
    player_cities: Query<&PlayerCities>,
    player_civ_cards: Query<&PlayerCivilizationCards>,
    area_query: Query<(Entity, &Population, Option<&BuiltCity>, Has<Volcano>, &LandPassage)>,
    volcano_areas: Query<Entity, With<Volcano>>,
    names: Query<&Name>,
) {
    for event in events.read() {
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

// ── Flood ─────────────────────────────────────────────────────────────────────

pub fn advance_flood(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution)>,
    flood_plains: Query<Entity, With<FloodPlain>>,
    area_query: Query<(Option<&BuiltCity>, &LandPassage)>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Flood(ref mut state) = *resolving else { continue };

        match state.phase {
            FloodPhase::FindFloodPlain => {
                let mut found_area = None;
                for fp_area in flood_plains.iter() {
                    if let Ok((Some(city), _)) = area_query.get(fp_area) {
                        if city.player == player_entity {
                            found_area = Some(fp_area);
                            break;
                        }
                    }
                }
                if let Some(area) = found_area {
                    state.flood_plain_area = Some(area);
                    state.phase = FloodPhase::ApplyEffects;
                } else {
                    info!("[FLOOD] No flood-plain city found for player {:?}", player_entity);
                    state.phase = FloodPhase::Complete;
                }
            }
            FloodPhase::ApplyEffects => {
                if let Some(fp_area) = state.flood_plain_area {
                    if state.has_engineering {
                        commands.entity(fp_area).insert(ReduceCity);
                    } else {
                        commands.entity(fp_area).insert(DestroyCity);
                        if let Ok((_, land_passage)) = area_query.get(fp_area) {
                            commands.entity(fp_area).insert(ClearAllTokens);
                            for &adj in &land_passage.to_areas {
                                commands.entity(adj).insert(ClearAllTokens);
                            }
                        }
                    }
                }
                state.phase = FloodPhase::Complete;
            }
            FloodPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::Flood);
            }
        }
    }
}

// ── Unit-point-loss calamities ────────────────────────────────────────────────

/// Removes `points` unit-points (tokens = 1 pt each) from the player's on-board token pool.
fn remove_unit_points(
    player: Entity,
    points: i32,
    player_areas: &PlayerAreas,
    populations: &mut Query<&mut Population>,
    commands: &mut Commands,
) {
    let mut remaining = points;
    for (area, count) in player_areas.areas_and_population_count() {
        if remaining <= 0 { break; }
        let to_remove = (remaining as usize).min(count);
        if to_remove == 0 { continue; }
        if let Ok(mut population) = populations.get_mut(area) {
            if let Some(removed) = population.remove_tokens_from_area(&player, to_remove) {
                let n = removed.len() as i32;
                for token in removed {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
                remaining -= n;
            }
        }
    }
}

pub fn advance_famine(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerAreas)>,
    mut populations: Query<&mut Population>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Famine(ref mut state) = *resolving else { continue };

        match state.phase {
            FaminePhase::ComputeLosses => {
                let loss = state.primary_loss;
                remove_unit_points(player_entity, loss, player_areas, &mut populations, &mut commands);
                // Secondary victim spread (TODO: interactive selection)
                info!("[FAMINE] Primary loss of {} applied; secondary spread not yet implemented", loss);
                state.phase = FaminePhase::Complete;
            }
            FaminePhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::Famine);
            }
            _ => {}
        }
    }
}

pub fn advance_barbarian_hordes(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerAreas)>,
    mut populations: Query<&mut Population>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::BarbarianHordes(ref mut state) = *resolving else { continue };

        match state.phase {
            BarbarianHordesPhase::ComputeEffects => {
                let loss = state.unit_points_to_lose;
                remove_unit_points(player_entity, loss, player_areas, &mut populations, &mut commands);
                // Area selection (TODO: interactive)
                info!("[BARBARIAN_HORDES] Loss of {} applied; area selection not yet implemented", loss);
                state.phase = BarbarianHordesPhase::Complete;
            }
            BarbarianHordesPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::BarbarianHordes);
            }
            _ => {}
        }
    }
}

pub fn advance_epidemic(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerAreas)>,
    mut populations: Query<&mut Population>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Epidemic(ref mut state) = *resolving else { continue };

        match state.phase {
            EpidemicPhase::ComputeEffects => {
                let loss = state.primary_loss;
                remove_unit_points(player_entity, loss, player_areas, &mut populations, &mut commands);
                state.phase = EpidemicPhase::ApplySecondaryLosses;
            }
            EpidemicPhase::ApplySecondaryLosses => {
                // TODO: spread secondary loss to adjacent players (immune player exempt)
                info!("[EPIDEMIC] Secondary spread not yet implemented");
                state.phase = EpidemicPhase::Complete;
            }
            EpidemicPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::Epidemic);
            }
            _ => {}
        }
    }
}

pub fn advance_iconoclasm_heresy(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerAreas)>,
    mut populations: Query<&mut Population>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::IconoclasmAndHeresy(ref mut state) = *resolving else { continue };

        match state.phase {
            IconoclasmHeresyPhase::ComputeEffects => {
                let loss = state.primary_loss;
                remove_unit_points(player_entity, loss, player_areas, &mut populations, &mut commands);
                state.phase = IconoclasmHeresyPhase::ApplySecondaryLosses;
            }
            IconoclasmHeresyPhase::ApplySecondaryLosses => {
                // TODO: secondary losses to non-immune players
                info!("[ICONOCLASM] Secondary losses not yet implemented");
                state.phase = IconoclasmHeresyPhase::Complete;
            }
            IconoclasmHeresyPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::IconoclasmAndHeresy);
            }
            _ => {}
        }
    }
}

// ── City-reduction calamities ─────────────────────────────────────────────────

pub fn advance_superstition(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerCities)>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Superstition(ref mut state) = *resolving else { continue };

        match state.phase {
            SuperstitionPhase::ComputeEffects => {
                // Auto-select up to cities_to_reduce cities (player choice TODO)
                let areas: Vec<Entity> = player_cities.areas_and_cities.keys()
                    .cloned()
                    .take(state.cities_to_reduce)
                    .collect();
                for area in areas {
                    state.select_city(area);
                    commands.entity(area).insert(ReduceCity);
                }
                info!("[SUPERSTITION] Reducing {} cities", state.selected_cities.len());
                state.phase = SuperstitionPhase::Complete;
            }
            SuperstitionPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::Superstition);
            }
            _ => {}
        }
    }
}

pub fn advance_slave_revolt(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerCities)>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::SlaveRevolt(ref mut state) = *resolving else { continue };

        match state.phase {
            SlaveRevoltPhase::ComputeEffects => {
                let areas: Vec<Entity> = player_cities.areas_and_cities.keys()
                    .cloned()
                    .take(state.cities_to_reduce)
                    .collect();
                for area in areas {
                    state.select_city(area);
                    commands.entity(area).insert(ReduceCity);
                }
                info!("[SLAVE_REVOLT] Reducing {} cities", state.selected_cities.len());
                state.phase = SlaveRevoltPhase::Complete;
            }
            SlaveRevoltPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::SlaveRevolt);
            }
            _ => {}
        }
    }
}

pub fn advance_civil_disorder(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerCities)>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::CivilDisorder(ref mut state) = *resolving else { continue };

        match state.phase {
            CivilDisorderPhase::ComputeEffects => {
                let areas: Vec<Entity> = player_cities.areas_and_cities.keys()
                    .cloned()
                    .take(state.cities_to_reduce)
                    .collect();
                for area in areas {
                    state.select_city(area);
                    commands.entity(area).insert(ReduceCity);
                }
                info!("[CIVIL_DISORDER] Reducing {} cities", state.selected_cities.len());
                state.phase = CivilDisorderPhase::Complete;
            }
            CivilDisorderPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::CivilDisorder);
            }
            _ => {}
        }
    }
}

// ── Multi-player calamities (partially implemented) ───────────────────────────

pub fn advance_civil_war(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution)>,
    all_players: Query<(Entity, &TokenStock), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::CivilWar(ref mut state) = *resolving else { continue };

        match state.phase {
            CivilWarPhase::DetermineBeneficiary => {
                // Beneficiary = other player with most tokens in stock
                let beneficiary = all_players.iter()
                    .filter(|(e, _)| *e != player_entity)
                    .max_by_key(|(_, stock)| stock.tokens_in_stock())
                    .map(|(e, _)| e);

                if let Some(b) = beneficiary {
                    info!("[CIVIL_WAR] Beneficiary is {:?}; victim must yield {} pts", b, state.victim_selection_points);
                    state.beneficiary = Some(b);
                    state.phase = CivilWarPhase::VictimSelectsUnits;
                } else {
                    info!("[CIVIL_WAR] No other players – no civil war effect");
                    state.phase = CivilWarPhase::Complete;
                }
            }
            CivilWarPhase::VictimSelectsUnits => {
                // TODO: player selects which units to hand over
                info!("[CIVIL_WAR] TODO – victim unit selection ({} pts)", state.victim_selection_points);
                state.phase = CivilWarPhase::BeneficiarySelectsUnits;
            }
            CivilWarPhase::BeneficiarySelectsUnits => {
                // TODO: beneficiary selects which units to take
                info!("[CIVIL_WAR] TODO – beneficiary unit selection (20 pts)");
                state.phase = CivilWarPhase::TransferFaction;
            }
            CivilWarPhase::TransferFaction => {
                // TODO: transfer tokens from victim's pool to beneficiary
                info!("[CIVIL_WAR] TODO – faction transfer");
                state.phase = CivilWarPhase::Complete;
            }
            CivilWarPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::CivilWar);
            }
        }
    }
}

pub fn advance_treachery(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution, &PlayerCities)>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Treachery(ref mut state) = *resolving else { continue };

        match state.phase {
            TreacheryPhase::SelectCity => {
                // Auto-select victim's first city (player choice TODO)
                let city_area = player_cities.areas_and_cities.keys().next().cloned();
                if let Some(area) = city_area {
                    state.city_to_replace = Some(area);
                    state.beneficiary = resolution.context.traded_by;
                    state.phase = TreacheryPhase::ApplyEffects;
                } else {
                    info!("[TREACHERY] No cities to treachery for player {:?}", player_entity);
                    state.phase = TreacheryPhase::Complete;
                }
            }
            TreacheryPhase::ApplyEffects => {
                if let Some(city_area) = state.city_to_replace {
                    if state.beneficiary.is_some() {
                        // TODO: transfer city to beneficiary (replace with their tokens)
                        info!("[TREACHERY] TODO – city transfer to beneficiary; destroying instead");
                    }
                    commands.entity(city_area).insert(DestroyCity);
                }
                state.phase = TreacheryPhase::Complete;
            }
            TreacheryPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::Treachery);
            }
        }
    }
}

pub fn advance_piracy(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ResolvingCalamity, &mut ActiveCalamityResolution)>,
    all_players: Query<(Entity, &TokenStock), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Piracy(ref mut state) = *resolving else { continue };

        match state.phase {
            PiracyPhase::DetermineBeneficiary => {
                let immune = resolution.context.traded_by;
                let beneficiary = all_players.iter()
                    .filter(|(e, _)| *e != player_entity && Some(*e) != immune)
                    .max_by_key(|(_, stock)| stock.tokens_in_stock())
                    .map(|(e, _)| e);

                if let Some(b) = beneficiary {
                    state.beneficiary = Some(b);
                    state.phase = PiracyPhase::SelectCoastalCities;
                } else {
                    state.phase = PiracyPhase::Complete;
                }
            }
            PiracyPhase::SelectCoastalCities => {
                // TODO: identify coastal cities (requires Coastal area marker)
                info!("[PIRACY] TODO – coastal city selection not implemented");
                state.phase = PiracyPhase::Complete;
            }
            PiracyPhase::ApplyEffects => {
                // TODO: replace coastal cities with beneficiary tokens
                info!("[PIRACY] TODO – apply piracy effects");
                state.phase = PiracyPhase::Complete;
            }
            PiracyPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::Piracy);
            }
        }
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn finish_calamity(
    resolution: &mut ActiveCalamityResolution,
    calamity_resolved: &mut MessageWriter<CalamityResolved>,
    commands: &mut Commands,
    player_entity: Entity,
    calamity: TradeCard,
) {
    resolution.mark_resolved();
    calamity_resolved.write(CalamityResolved { player: player_entity, calamity });
    commands.entity(player_entity).remove::<ResolvingCalamity>();
    commands.entity(player_entity).remove::<ActiveCalamityResolution>();
}

// ── Post-resolution cleanup ───────────────────────────────────────────────────

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

// ── Marker components and their systems ──────────────────────────────────────

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
