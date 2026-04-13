use bevy::prelude::*;
use rand::rng;
use rand::seq::SliceRandom;

use crate::GameActivity;
use crate::civilization::components::*;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::resolve_calamities::calamities::ResolvingCalamity;
use crate::civilization::concepts::resolve_calamities::calamities::barbarian_hordes::{
    BarbarianHordesPhase, BarbarianHordesState,
};
use crate::civilization::concepts::resolve_calamities::calamities::civil_disorder::{
    CivilDisorderPhase, CivilDisorderState,
};
use crate::civilization::concepts::resolve_calamities::calamities::civil_war::{
    CivilWarPhase, CivilWarState,
};
use crate::civilization::concepts::resolve_calamities::calamities::epidemic::{
    EpidemicPhase, EpidemicState,
};
use crate::civilization::concepts::resolve_calamities::calamities::famine::{
    FaminePhase, FamineState,
};
use crate::civilization::concepts::resolve_calamities::calamities::flood::{
    FloodPhase, FloodState,
};
use crate::civilization::concepts::resolve_calamities::calamities::iconoclasm_heresy::{
    IconoclasmHeresyPhase, IconoclasmHeresyState,
};
use crate::civilization::concepts::resolve_calamities::calamities::piracy::{
    PiracyPhase, PiracyState,
};
use crate::civilization::concepts::resolve_calamities::calamities::slave_revolt::{
    SlaveRevoltPhase, SlaveRevoltState,
};
use crate::civilization::concepts::resolve_calamities::calamities::superstition::{
    SuperstitionPhase, SuperstitionState,
};
use crate::civilization::concepts::resolve_calamities::calamities::treachery::{
    TreacheryPhase, TreacheryState,
};
use crate::civilization::concepts::resolve_calamities::calamities::volcano_earthquake::{
    VolcanoEarthquakePhase, VolcanoEarthquakeState,
};
use crate::civilization::concepts::resolve_calamities::context::{
    ActiveCalamityResolution, CalamityContext, CalamityPhase,
};
use crate::civilization::concepts::resolve_calamities::resolve_calamities_components::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_events::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
    AwaitingHumanCalamitySelection, CalamitySelectionState, CivilWarSelectionState,
};
use crate::civilization::functions::return_all_tokens_to_stock;
use crate::civilization::{CivCardName, PlayerTradeCards, TradeCard, TradeCardTrait};
use crate::player::Player;
use crate::stupid_ai::IsHuman;

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
        info!(
            "[CALAMITIES] Player {:?} has {} calamities",
            player_entity,
            calamity_cards.len()
        );

        let calamities_to_resolve = if calamity_cards.len() > 2 {
            select_random_calamities(&calamity_cards, 2)
        } else {
            calamity_cards.iter().map(|c| (*c, None)).collect()
        };

        commands
            .entity(player_entity)
            .insert(PendingCalamities::new(calamities_to_resolve));
        commands
            .entity(player_entity)
            .insert(NeedsCalamityResolution);
    }

    if !any_calamities {
        info!("[CALAMITIES] No calamities to resolve, skipping phase");
        next_state.set(GameActivity::CheckCitySupportAfterResolveCalamities);
    }
}

fn select_random_calamities(
    calamities: &[TradeCard],
    count: usize,
) -> Vec<(TradeCard, Option<Entity>)> {
    let mut rng = rng();
    let mut shuffled: Vec<TradeCard> = calamities.to_vec();
    shuffled.shuffle(&mut rng);
    shuffled
        .into_iter()
        .take(count)
        .map(|c| (c, None))
        .collect()
}

/// Picks the next calamity to resolve (globally lowest value, non-tradeable first) and dispatches
/// it by attaching `ResolvingCalamity` + `ActiveCalamityResolution` to the victim entity.
/// Only runs when no calamity is currently being resolved.
pub fn process_pending_calamities(
    mut commands: Commands,
    mut players_with_pending: Query<
        (Entity, &mut PendingCalamities, &mut PlayerTradeCards),
        With<NeedsCalamityResolution>,
    >,
    player_civ_cards: Query<&PlayerCivilizationCards>,
    all_players_civ: Query<(Entity, &PlayerCivilizationCards), With<Player>>,
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
            commands
                .entity(player_entity)
                .remove::<NeedsCalamityResolution>();
            commands.entity(player_entity).remove::<PendingCalamities>();
        }
        // Rule 32.94: Monotheism holders convert adjacent enemy units at end of phase.
        // Mark each Monotheism holder with NeedsMonotheismConversion; the
        // apply_monotheism_conversions system handles the actual transition once done.
        let monotheism_holders: Vec<Entity> = all_players_civ
            .iter()
            .filter(|(_, c)| c.owns(&CivCardName::Monotheism))
            .map(|(e, _)| e)
            .collect();
        if monotheism_holders.is_empty() {
            info!("[CALAMITIES] All calamities resolved, transitioning to CheckCitySupport");
            next_state.set(GameActivity::CheckCitySupportAfterResolveCalamities);
        } else {
            info!(
                "[MONOTHEISM] {} Monotheism holder(s) pending conversion",
                monotheism_holders.len()
            );
            for e in monotheism_holders {
                commands.entity(e).insert(NeedsMonotheismConversion);
            }
        }
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
        info!(
            "[CALAMITIES] Resolving {:?} for player {:?}",
            calamity, player_entity
        );

        let civ_cards = player_civ_cards.get(*player_entity).ok();

        if let Ok((_, mut pending, mut trade_cards)) = players_with_pending.get_mut(*player_entity)
        {
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
                    let has_pottery = civ_cards
                        .map(|c| c.owns(&CivCardName::Pottery))
                        .unwrap_or(false);
                    let state = FamineState::new().with_grain_reduction(grain_count, has_pottery);
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::Famine(state),
                    ));
                }
                TradeCard::Superstition => {
                    let state = if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Enlightenment) {
                            SuperstitionState::new().with_enlightenment()
                        } else if c.owns(&CivCardName::Deism) {
                            SuperstitionState::new().with_deism()
                        } else if c.owns(&CivCardName::Mysticism) {
                            SuperstitionState::new().with_mysticism()
                        } else {
                            SuperstitionState::new()
                        }
                    } else {
                        SuperstitionState::new()
                    };
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::Superstition(state),
                    ));
                }
                TradeCard::CivilWar => {
                    let mut state = CivilWarState::new();
                    if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Music) {
                            state.apply_music_bonus();
                        }
                        if c.owns(&CivCardName::DramaAndPoetry) {
                            state.apply_drama_poetry_bonus();
                        }
                        if c.owns(&CivCardName::Democracy) {
                            state.apply_democracy_bonus();
                        }
                        // Rule 31.74: Philosophy reduces the victim's point loss by 5.
                        if c.owns(&CivCardName::Philosophy) {
                            state.apply_philosophy_protection();
                        }
                    }
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::CivilWar(state),
                    ));
                }
                TradeCard::SlaveRevolt => {
                    // 15 tokens can't support cities (30.421); Mining +5, Enlightenment -5, both cancel (30.423).
                    let has_mining = civ_cards
                        .map(|c| c.owns(&CivCardName::Mining))
                        .unwrap_or(false);
                    let has_enlightenment = civ_cards
                        .map(|c| c.owns(&CivCardName::Enlightenment))
                        .unwrap_or(false);
                    let state = match (has_mining, has_enlightenment) {
                        (true, true) => SlaveRevoltState::new().with_mining_and_enlightenment(),
                        (true, false) => SlaveRevoltState::new().with_mining(),
                        (false, true) => SlaveRevoltState::new().with_enlightenment(),
                        (false, false) => SlaveRevoltState::new(),
                    };
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::SlaveRevolt(state),
                    ));
                }
                TradeCard::Flood => {
                    let has_engineering = civ_cards
                        .map(|c| c.owns(&CivCardName::Engineering))
                        .unwrap_or(false);
                    let mut state = FloodState::new();
                    if has_engineering {
                        state = state.with_engineering();
                    }
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::Flood(state),
                    ));
                }
                TradeCard::BarbarianHordes => {
                    let has_military = civ_cards
                        .map(|c| c.owns(&CivCardName::Military))
                        .unwrap_or(false);
                    let state = if has_military {
                        BarbarianHordesState::new().with_military()
                    } else {
                        BarbarianHordesState::new()
                    };
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::BarbarianHordes(state),
                    ));
                }
                TradeCard::Epidemic => {
                    let has_medicine = civ_cards
                        .map(|c| c.owns(&CivCardName::Medicine))
                        .unwrap_or(false);
                    let has_road_building = civ_cards
                        .map(|c| c.owns(&CivCardName::RoadBuilding))
                        .unwrap_or(false);
                    let mut state = EpidemicState::new();
                    if has_medicine {
                        state = state.with_medicine();
                    }
                    if has_road_building {
                        state = state.with_road_building();
                    }
                    if let Some(immune) = *traded_by {
                        state = state.with_immune_player(immune);
                    }
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::Epidemic(state),
                    ));
                }
                TradeCard::CivilDisorder => {
                    // Default: all but 3 cities reduced (30.711); modifiers cumulative (30.715).
                    let mut state = CivilDisorderState::new();
                    if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Music) {
                            state = state.with_music();
                        }
                        if c.owns(&CivCardName::DramaAndPoetry) {
                            state = state.with_drama_and_poetry();
                        }
                        if c.owns(&CivCardName::Law) {
                            state = state.with_law();
                        }
                        if c.owns(&CivCardName::Democracy) {
                            state = state.with_democracy();
                        }
                        if c.owns(&CivCardName::Military) {
                            state = state.with_military();
                        }
                        if c.owns(&CivCardName::RoadBuilding) {
                            state = state.with_road_building();
                        }
                    }
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::CivilDisorder(state),
                    ));
                }
                TradeCard::IconoclasmAndHeresy => {
                    // 4 cities reduced by default (30.811); all modifiers cumulative (30.817).
                    let mut state = IconoclasmHeresyState::new();
                    if let Some(c) = civ_cards {
                        if c.owns(&CivCardName::Law) {
                            state = state.with_law();
                        }
                        if c.owns(&CivCardName::Philosophy) {
                            state = state.with_philosophy();
                        }
                        if c.owns(&CivCardName::Theology) {
                            state = state.with_theology();
                        }
                        if c.owns(&CivCardName::Monotheism) {
                            state = state.with_monotheism();
                        }
                        if c.owns(&CivCardName::RoadBuilding) {
                            state = state.with_road_building();
                        }
                    }
                    if let Some(immune) = *traded_by {
                        state = state.with_immune_player(immune);
                    }
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::IconoclasmAndHeresy(state),
                    ));
                }
                TradeCard::Treachery => {
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::Treachery(TreacheryState::new()),
                    ));
                }
                TradeCard::Piracy => {
                    let mut state = PiracyState::new();
                    if let Some(immune) = *traded_by {
                        state = state.with_immune_player(immune);
                    }
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::Piracy(state),
                    ));
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
    area_query: Query<(
        Entity,
        &Population,
        Option<&BuiltCity>,
        Has<Volcano>,
        &LandPassage,
    )>,
    volcano_areas: Query<Entity, With<Volcano>>,
    names: Query<&Name>,
) {
    for event in events.read() {
        let primary_victim = event.primary_victim;
        let player_name = names
            .get(primary_victim)
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        info!("[VOLCANO/EARTHQUAKE] Resolving for {}", player_name);

        let has_engineering = player_civ_cards
            .get(primary_victim)
            .map(|cards: &PlayerCivilizationCards| cards.owns(&CivCardName::Engineering))
            .unwrap_or(false);

        let player_cities_component = player_cities.get(primary_victim).ok();

        let volcano_result =
            find_best_volcano_eruption(primary_victim, &volcano_areas, &area_query);

        let state = if let Some((volcano_area, areas_to_clear)) = volcano_result {
            info!("[VOLCANO] Eruption at area {:?}", volcano_area);
            VolcanoEarthquakeState::as_volcano(volcano_area, areas_to_clear)
        } else {
            info!("[EARTHQUAKE] No volcano areas with cities, triggering earthquake");

            if let Some(cities) = player_cities_component {
                let city_areas: Vec<Entity> = cities.areas_and_cities.keys().cloned().collect();

                if !city_areas.is_empty() {
                    let city_to_destroy = city_areas[0];
                    let city_to_reduce =
                        find_adjacent_city_to_reduce(city_to_destroy, primary_victim, &area_query);

                    info!(
                        "[EARTHQUAKE] City to destroy: {:?}, city to reduce: {:?}, has_engineering: {}",
                        city_to_destroy, city_to_reduce, has_engineering
                    );

                    VolcanoEarthquakeState::as_earthquake(
                        city_to_destroy,
                        city_to_reduce,
                        has_engineering,
                    )
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
    area_query: &Query<(
        Entity,
        &Population,
        Option<&BuiltCity>,
        Has<Volcano>,
        &LandPassage,
    )>,
) -> Option<(Entity, Vec<Entity>)> {
    let mut volcano_candidates: Vec<(Entity, usize, Vec<Entity>)> = Vec::new();

    for volcano_area in volcano_areas.iter() {
        if let Ok((_area_entity, population, built_city, _, land_passage)) =
            area_query.get(volcano_area)
        {
            let mut total_damage = 0usize;
            let mut victim_has_city_in_touched_areas = false;
            let mut areas_to_clear = vec![volcano_area];

            if let Some(city) = built_city
                && city.player == primary_victim
            {
                total_damage += 5;
                victim_has_city_in_touched_areas = true;
            }

            if let Some(tokens) = population.tokens_for_player(&primary_victim) {
                total_damage += tokens.len();
            }

            for adjacent_area in land_passage.to_areas.iter() {
                areas_to_clear.push(*adjacent_area);
                if let Ok((_, adj_pop, adj_city, _, _)) = area_query.get(*adjacent_area) {
                    if let Some(city) = adj_city
                        && city.player == primary_victim
                    {
                        total_damage += 5;
                        victim_has_city_in_touched_areas = true;
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
    area_query: &Query<(
        Entity,
        &Population,
        Option<&BuiltCity>,
        Has<Volcano>,
        &LandPassage,
    )>,
) -> Option<Entity> {
    if let Ok((_, _, _, _, land_passage)) = area_query.get(city_area) {
        for adjacent_area in land_passage.to_areas.iter() {
            if let Ok((_, _, Some(adj_city), _, _)) = area_query.get(*adjacent_area)
                && adj_city.player != primary_victim
            {
                return Some(*adjacent_area);
            }
        }
    }
    None
}

pub fn apply_volcano_earthquake_effects(
    mut commands: Commands,
    mut players_resolving: Query<(Entity, &mut ActiveCalamityResolution, &ResolvingCalamity)>,
    area_query: Query<(
        Entity,
        &Population,
        Option<&BuiltCity>,
        Has<Volcano>,
        &LandPassage,
    )>,
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
            commands
                .entity(player_entity)
                .remove::<ActiveCalamityResolution>();
        }
    }
}

fn trigger_volcano_eruption(
    commands: &mut Commands,
    volcano_area: Entity,
    area_query: &Query<(
        Entity,
        &Population,
        Option<&BuiltCity>,
        Has<Volcano>,
        &LandPassage,
    )>,
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
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
    )>,
    flood_plains: Query<Entity, With<FloodPlain>>,
    area_query: Query<(Option<&BuiltCity>, &LandPassage)>,
    mut populations: Query<&mut Population>,
    sea_passage_query: Query<Has<SeaPassage>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::Flood(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            FloodPhase::FindFloodPlain => {
                // Rule 30.51: pick the flood plain where the victim has the most unit points.
                let mut best_area: Option<Entity> = None;
                let mut best_pts = 0usize;
                for fp_area in flood_plains.iter() {
                    let pts = populations
                        .get(fp_area)
                        .map(|pop| pop.population_for_player(player_entity))
                        .unwrap_or(0);
                    if pts > best_pts {
                        best_pts = pts;
                        best_area = Some(fp_area);
                    }
                }
                if let Some(area) = best_area {
                    state.flood_plain_area = Some(area);
                    info!(
                        "[FLOOD] Flood plain chosen for {:?}: {:?} ({} victim pts)",
                        player_entity, area, best_pts
                    );
                    state.phase = FloodPhase::ApplyPrimaryLoss;
                } else {
                    info!(
                        "[FLOOD] No flood-plain units found for player {:?}, using fallback",
                        player_entity
                    );
                    state.phase = FloodPhase::FallbackCoastalCity;
                }
            }
            FloodPhase::ApplyPrimaryLoss => {
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
                state.phase = FloodPhase::ApplySecondaryLoss;
            }
            FloodPhase::ApplySecondaryLoss => {
                // Rule 30.51: secondary victims on the same flood plain collectively lose 10 pts.
                // Distribute evenly across secondary victims present, removing from the flood area.
                if let Some(fp_area) = state.flood_plain_area {
                    if let Ok(mut pop) = populations.get_mut(fp_area) {
                        let secondary_players: Vec<Entity> = pop
                            .player_tokens()
                            .keys()
                            .filter(|&&e| e != player_entity)
                            .cloned()
                            .collect();
                        let n = secondary_players.len();
                        if n > 0 {
                            let total_loss = 10usize;
                            let per_player = total_loss.div_ceil(n);
                            for sec in secondary_players {
                                let available = pop.population_for_player(sec);
                                let to_remove = per_player.min(available);
                                if let Some(removed) = pop.remove_tokens_from_area(&sec, to_remove) {
                                    for token in removed {
                                        commands.entity(token).insert(ReturnTokenToStock);
                                    }
                                }
                            }
                            info!("[FLOOD] Applied secondary loss of {} pts ({} secondary victims)", total_loss, n);
                        }
                    }
                }
                state.phase = FloodPhase::Complete;
            }
            FloodPhase::FallbackCoastalCity => {
                // Rule 30.51: if no flood plain had victim units, eliminate one coastal city.
                // With Engineering: reduce instead of destroy.
                let coastal_city_area = player_cities
                    .areas_and_cities
                    .keys()
                    .find(|&&area| sea_passage_query.get(area).unwrap_or(false))
                    .or_else(|| player_cities.areas_and_cities.keys().next())
                    .cloned();
                if let Some(area) = coastal_city_area {
                    if state.has_engineering {
                        commands.entity(area).insert(ReduceCity);
                    } else {
                        commands.entity(area).insert(DestroyCity);
                    }
                    info!("[FLOOD] Fallback: eliminated coastal city in area {:?}", area);
                }
                state.phase = FloodPhase::Complete;
            }
            FloodPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::Flood,
                );
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
        if remaining <= 0 {
            break;
        }
        let to_remove = (remaining as usize).min(count);
        if to_remove == 0 {
            continue;
        }
        if let Ok(mut population) = populations.get_mut(area)
            && let Some(removed) = population.remove_tokens_from_area(&player, to_remove)
        {
            let n = removed.len() as i32;
            for token in removed {
                commands.entity(token).insert(ReturnTokenToStock);
            }
            remaining -= n;
        }
    }
}

pub fn advance_famine(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerAreas,
    )>,
    mut populations: Query<&mut Population>,
    all_players: Query<(Entity, &PlayerAreas), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::Famine(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            FaminePhase::ComputeLosses => {
                let loss = state.primary_loss;
                remove_unit_points(
                    player_entity,
                    loss,
                    player_areas,
                    &mut populations,
                    &mut commands,
                );
                info!("[FAMINE] Primary loss of {} applied", loss);
                state.phase = FaminePhase::ApplySecondaryLosses;
            }
            FaminePhase::ApplySecondaryLosses => {
                // Secondary victims: players sharing areas with primary victim lose up to 20 pts
                // total, max 8 per player (rule 30.311). Auto-distributed.
                let primary_areas: bevy::platform::collections::HashSet<Entity> =
                    player_areas.areas().iter().cloned().collect();

                let max_per_player = 8i32;
                let mut remaining = 20i32;

                let secondary: Vec<Entity> = all_players
                    .iter()
                    .filter(|(e, areas)| {
                        *e != player_entity
                            && areas.areas().iter().any(|a| primary_areas.contains(a))
                    })
                    .map(|(e, _)| e)
                    .collect();

                for secondary_entity in secondary {
                    if remaining <= 0 {
                        break;
                    }
                    let loss = max_per_player.min(remaining);
                    if let Ok((_, sec_areas)) = all_players.get(secondary_entity) {
                        remove_unit_points(
                            secondary_entity,
                            loss,
                            sec_areas,
                            &mut populations,
                            &mut commands,
                        );
                        remaining -= loss;
                        info!(
                            "[FAMINE] Secondary player {:?} loses {} pts",
                            secondary_entity, loss
                        );
                    }
                }

                state.phase = FaminePhase::Complete;
            }
            FaminePhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::Famine,
                );
            }
            _ => {}
        }
    }
}

pub fn advance_barbarian_hordes(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &Faction,
    )>,
    start_areas: Query<(Entity, &StartArea, &LandPassage)>,
    mut populations: Query<&mut Population>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, faction) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::BarbarianHordes(ref mut state) = *resolving else { continue };

        match state.phase {
            BarbarianHordesPhase::FindLandingArea => {
                // Rule 30.527: Crete may not be primary victim.
                if faction.faction == crate::civilization::enums::GameFaction::Crete {
                    info!("[BARBARIAN_HORDES] Crete is immune (30.527)");
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                }

                // Rule 30.5211: place in victim's start area causing greatest damage
                // (most victim unit points). If no units in any start area, use empty start area.
                let victim_faction = faction.faction;
                let mut best_area: Option<Entity> = None;
                let mut best_pts = 0usize;

                for (area_entity, start_area, _) in start_areas.iter() {
                    if start_area.faction != victim_faction { continue; }
                    let victim_pts = populations.get(area_entity)
                        .map(|pop| pop.population_for_player(player_entity))
                        .unwrap_or(0);
                    if victim_pts > best_pts || best_area.is_none() {
                        best_pts = victim_pts;
                        best_area = Some(area_entity);
                    }
                }

                state.landing_area = best_area;
                info!(
                    "[BARBARIAN_HORDES] {:?} landing in {:?} ({}  victim units there)",
                    victim_faction, best_area, best_pts
                );
                state.phase = BarbarianHordesPhase::ApplyEffects;
            }
            BarbarianHordesPhase::ApplyEffects => {
                let loss = state.unit_points_to_lose;

                if let Some(landing) = state.landing_area {
                    // Rule 30.5211/30.5231: remove tokens from landing area first,
                    // then cascade into adjacent areas (simulates Barbarian advance).
                    let mut remaining = loss;

                    // Remove from landing area
                    if let Ok(mut pop) = populations.get_mut(landing) {
                        let available = pop.population_for_player(player_entity) as i32;
                        let to_remove = available.min(remaining);
                        if to_remove > 0 {
                            if let Some(removed) = pop.remove_tokens_from_area(&player_entity, to_remove as usize) {
                                for token in removed { commands.entity(token).insert(ReturnTokenToStock); }
                                remaining -= to_remove;
                            }
                        }
                    }

                    // Continue into adjacent areas
                    if remaining > 0 {
                        let adjacent: Vec<Entity> = start_areas.get(landing)
                            .map(|(_, _, lp)| lp.to_areas.clone())
                            .unwrap_or_default();

                        for adj_area in adjacent {
                            if remaining <= 0 { break; }
                            if let Ok(mut pop) = populations.get_mut(adj_area) {
                                let available = pop.population_for_player(player_entity) as i32;
                                let to_remove = available.min(remaining);
                                if to_remove > 0 {
                                    if let Some(removed) = pop.remove_tokens_from_area(&player_entity, to_remove as usize) {
                                        for token in removed { commands.entity(token).insert(ReturnTokenToStock); }
                                        remaining -= to_remove;
                                    }
                                }
                            }
                        }
                    }

                    // If still short (victim had fewer units than loss), nothing more to do
                    info!("[BARBARIAN_HORDES] Applied {} unit points of loss", loss - remaining);
                } else {
                    // No start area found — fall back to flat removal
                    info!("[BARBARIAN_HORDES] No start area found, applying flat loss of {}", loss);
                }

                state.phase = BarbarianHordesPhase::Complete;
            }
            BarbarianHordesPhase::Complete => {
                finish_calamity(&mut resolution, &mut calamity_resolved, &mut commands, player_entity, TradeCard::BarbarianHordes);
            }
        }
    }
}

pub fn advance_epidemic(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerAreas,
    )>,
    mut populations: Query<&mut Population>,
    all_players: Query<(Entity, &PlayerAreas, Option<&PlayerCivilizationCards>), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::Epidemic(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            EpidemicPhase::ComputeEffects => {
                let loss = state.primary_loss;
                remove_unit_points(
                    player_entity,
                    loss,
                    player_areas,
                    &mut populations,
                    &mut commands,
                );
                state.phase = EpidemicPhase::ApplySecondaryLosses;
            }
            EpidemicPhase::ApplySecondaryLosses => {
                // Spread secondary_loss (total 25) across players sharing areas with the primary
                // victim, max 10 per player (rule 30.612). Immune player (trader) is exempt.
                let primary_areas: bevy::platform::collections::HashSet<Entity> =
                    player_areas.areas().iter().cloned().collect();

                let secondary_total = state.secondary_loss;
                let max_per_player = 10i32;
                let mut remaining = secondary_total;

                let secondary_players: Vec<(Entity, bool)> = all_players
                    .iter()
                    .filter(|(e, areas, _)| {
                        *e != player_entity
                            && Some(*e) != state.immune_player
                            && areas.areas().iter().any(|a| primary_areas.contains(a))
                    })
                    .map(|(e, _, cards)| {
                        let has_medicine = cards
                            .map(|c| c.owns(&CivCardName::Medicine))
                            .unwrap_or(false);
                        (e, has_medicine)
                    })
                    .collect();

                for (secondary_entity, has_medicine) in &secondary_players {
                    if remaining <= 0 {
                        break;
                    }
                    let mut loss = max_per_player.min(remaining);
                    if *has_medicine {
                        loss = (loss - 5).max(0);
                    }
                    if let Ok((_, sec_areas, _)) = all_players.get(*secondary_entity) {
                        remove_unit_points(
                            *secondary_entity,
                            loss,
                            sec_areas,
                            &mut populations,
                            &mut commands,
                        );
                        remaining -= loss;
                        info!(
                            "[EPIDEMIC] Secondary player {:?} loses {} pts",
                            secondary_entity, loss
                        );
                    }
                }

                state.phase = EpidemicPhase::Complete;
            }
            EpidemicPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::Epidemic,
                );
            }
            _ => {}
        }
    }
}

pub fn advance_iconoclasm_heresy(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    all_players: Query<(Entity, &PlayerCities, Option<&PlayerCivilizationCards>), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, awaiting_human) in
        player_query.iter_mut()
    {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::IconoclasmAndHeresy(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            IconoclasmHeresyPhase::ComputeEffects => {
                info!(
                    "[ICONOCLASM] Primary victim reducing {} cities",
                    state.cities_to_reduce
                );
                if state.cities_to_reduce == 0 {
                    state.phase = IconoclasmHeresyPhase::ApplySecondaryLosses;
                } else if is_human {
                    let available: Vec<Entity> =
                        player_cities.areas_and_cities.keys().cloned().collect();
                    calamity_selection.populate(
                        player_entity,
                        available,
                        state.cities_to_reduce,
                        "Iconoclasm & Heresy",
                    );
                    commands
                        .entity(player_entity)
                        .insert(AwaitingHumanCalamitySelection);
                    state.phase = IconoclasmHeresyPhase::SelectCities;
                } else {
                    let areas: Vec<Entity> = player_cities
                        .areas_and_cities
                        .keys()
                        .cloned()
                        .take(state.cities_to_reduce)
                        .collect();
                    for area in &areas {
                        state.select_city(*area);
                    }
                    state.phase = IconoclasmHeresyPhase::ApplySecondaryLosses;
                }
            }
            IconoclasmHeresyPhase::SelectCities => if !awaiting_human {
                    for area in calamity_selection.take_selected_cities() {
                        state.select_city(area);
                    }
                    state.phase = IconoclasmHeresyPhase::ApplySecondaryLosses;
                }
            

            IconoclasmHeresyPhase::ApplySecondaryLosses => {
                // Apply primary reductions
                for &area in &state.selected_cities {
                    commands.entity(area).insert(ReduceCity);
                }

                // Primary victim orders 2 secondary cities reduced from other players (30.818).
                // Auto-select: Theology holders are immune; Philosophy holders lose max 1.
                let mut secondary_cities_left = state.secondary_cities;

                let candidates: Vec<(Entity, Vec<Entity>, bool)> = all_players
                    .iter()
                    .filter(|(e, cities, _)| {
                        *e != player_entity
                            && Some(*e) != state.immune_player
                            && !cities.areas_and_cities.is_empty()
                    })
                    .filter_map(|(e, cities, cards)| {
                        let has_theology = cards
                            .map(|c| c.owns(&CivCardName::Theology))
                            .unwrap_or(false);
                        if has_theology {
                            return None;
                        } // Theology immune (30.819)
                        let has_philosophy = cards
                            .map(|c| c.owns(&CivCardName::Philosophy))
                            .unwrap_or(false);
                        let city_areas: Vec<Entity> =
                            cities.areas_and_cities.keys().cloned().collect();
                        Some((e, city_areas, has_philosophy))
                    })
                    .collect();

                for (_secondary_entity, city_areas, has_philosophy) in &candidates {
                    if secondary_cities_left == 0 {
                        break;
                    }
                    let max_from_this = if *has_philosophy {
                        1
                    } else {
                        secondary_cities_left
                    };
                    let to_take = max_from_this.min(secondary_cities_left);
                    for &area in city_areas.iter().take(to_take) {
                        commands.entity(area).insert(ReduceCity);
                        secondary_cities_left -= 1;
                    }
                }

                info!("[ICONOCLASM] Secondary reductions applied");
                state.phase = IconoclasmHeresyPhase::Complete;
            }
            IconoclasmHeresyPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::IconoclasmAndHeresy,
                );
            }
            _ => {}
        }
    }
}

// ── City-reduction calamities ─────────────────────────────────────────────────

pub fn advance_superstition(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, awaiting_human) in
        player_query.iter_mut()
    {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::Superstition(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            SuperstitionPhase::ComputeEffects => {
                if state.cities_to_reduce == 0 {
                    state.phase = SuperstitionPhase::Complete;
                } else if is_human {
                    let available: Vec<Entity> =
                        player_cities.areas_and_cities.keys().cloned().collect();
                    calamity_selection.populate(
                        player_entity,
                        available,
                        state.cities_to_reduce,
                        "Superstition",
                    );
                    commands
                        .entity(player_entity)
                        .insert(AwaitingHumanCalamitySelection);
                    state.phase = SuperstitionPhase::SelectCities;
                } else {
                    let areas: Vec<Entity> = player_cities
                        .areas_and_cities
                        .keys()
                        .cloned()
                        .take(state.cities_to_reduce)
                        .collect();
                    for area in &areas {
                        state.select_city(*area);
                    }
                    state.phase = SuperstitionPhase::ApplyEffects;
                }
            }
            SuperstitionPhase::SelectCities => {
                if !awaiting_human {
                    // Human confirmed their selection
                    for area in calamity_selection.take_selected_cities() {
                        state.select_city(area);
                    }
                    state.phase = SuperstitionPhase::ApplyEffects;
                }
                // else: still waiting for UI
            }
            SuperstitionPhase::ApplyEffects => {
                for &area in &state.selected_cities {
                    commands.entity(area).insert(ReduceCity);
                }
                info!(
                    "[SUPERSTITION] Reducing {} cities",
                    state.selected_cities.len()
                );
                state.phase = SuperstitionPhase::Complete;
            }
            SuperstitionPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::Superstition,
                );
            }
        }
    }
}

pub fn advance_slave_revolt(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
        &PlayerAreas,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (
        player_entity,
        mut resolving,
        mut resolution,
        player_cities,
        player_areas,
        is_human,
        awaiting_human,
    ) in player_query.iter_mut()
    {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::SlaveRevolt(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            SlaveRevoltPhase::ComputeEffects => {
                let on_board_tokens = player_areas.total_population();
                let city_count = player_cities.number_of_cities();
                state.compute_cities_to_reduce(on_board_tokens, city_count);
                info!(
                    "[SLAVE_REVOLT] {} tokens cannot support cities; need to reduce {} cities",
                    state.tokens_cannot_support, state.cities_to_reduce
                );

                if state.cities_to_reduce == 0 {
                    state.phase = SlaveRevoltPhase::Complete;
                } else if is_human {
                    let available: Vec<Entity> =
                        player_cities.areas_and_cities.keys().cloned().collect();
                    calamity_selection.populate(
                        player_entity,
                        available,
                        state.cities_to_reduce,
                        "Slave Revolt",
                    );
                    commands
                        .entity(player_entity)
                        .insert(AwaitingHumanCalamitySelection);
                    state.phase = SlaveRevoltPhase::SelectCities;
                } else {
                    let areas: Vec<Entity> = player_cities
                        .areas_and_cities
                        .keys()
                        .cloned()
                        .take(state.cities_to_reduce)
                        .collect();
                    for area in &areas {
                        state.select_city(*area);
                    }
                    state.phase = SlaveRevoltPhase::ApplyEffects;
                }
            }
            SlaveRevoltPhase::SelectCities => {
                if !awaiting_human {
                    for area in calamity_selection.take_selected_cities() {
                        state.select_city(area);
                    }
                    state.phase = SlaveRevoltPhase::ApplyEffects;
                }
            }
            SlaveRevoltPhase::ApplyEffects => {
                for &area in &state.selected_cities {
                    commands.entity(area).insert(ReduceCity);
                }
                info!(
                    "[SLAVE_REVOLT] Reducing {} cities",
                    state.selected_cities.len()
                );
                state.phase = SlaveRevoltPhase::Complete;
            }
            SlaveRevoltPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::SlaveRevolt,
                );
            }
        }
    }
}

pub fn advance_civil_disorder(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, awaiting_human) in
        player_query.iter_mut()
    {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::CivilDisorder(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            CivilDisorderPhase::ComputeEffects => {
                let total_cities = player_cities.number_of_cities();
                state.compute_cities_to_reduce(total_cities);
                info!(
                    "[CIVIL_DISORDER] Need to reduce {} cities",
                    state.cities_to_reduce
                );

                if state.cities_to_reduce == 0 {
                    state.phase = CivilDisorderPhase::Complete;
                } else if is_human {
                    let available: Vec<Entity> =
                        player_cities.areas_and_cities.keys().cloned().collect();
                    calamity_selection.populate(
                        player_entity,
                        available,
                        state.cities_to_reduce,
                        "Civil Disorder",
                    );
                    commands
                        .entity(player_entity)
                        .insert(AwaitingHumanCalamitySelection);
                    state.phase = CivilDisorderPhase::SelectCities;
                } else {
                    let areas: Vec<Entity> = player_cities
                        .areas_and_cities
                        .keys()
                        .cloned()
                        .take(state.cities_to_reduce)
                        .collect();
                    for area in &areas {
                        state.select_city(*area);
                    }
                    state.phase = CivilDisorderPhase::ApplyEffects;
                }
            }
            CivilDisorderPhase::SelectCities => {
                if !awaiting_human {
                    for area in calamity_selection.take_selected_cities() {
                        state.select_city(area);
                    }
                    state.phase = CivilDisorderPhase::ApplyEffects;
                }
            }
            CivilDisorderPhase::ApplyEffects => {
                for &area in &state.selected_cities {
                    commands.entity(area).insert(ReduceCity);
                }
                info!(
                    "[CIVIL_DISORDER] Reducing {} cities",
                    state.selected_cities.len()
                );
                state.phase = CivilDisorderPhase::Complete;
            }
            CivilDisorderPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::CivilDisorder,
                );
            }
        }
    }
}

// ── Multi-player calamities ───────────────────────────────────────────────────

pub fn advance_civil_war(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerAreas,
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    all_players_stock: Query<(Entity, &TokenStock), With<Player>>,
    all_players_civ_cards: Query<(Entity, &PlayerCivilizationCards), With<Player>>,
    human_query: Query<Entity, With<IsHuman>>,
    beneficiary_waiting: Query<Entity, With<AwaitingHumanCalamitySelection>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut cw_selection: ResMut<CivilWarSelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, victim_areas, victim_cities, victim_is_human, victim_awaiting) in
        player_query.iter_mut()
    {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::CivilWar(ref mut state) = *resolving else { continue };

        match state.phase {
            CivilWarPhase::DetermineBeneficiary => {
                let beneficiary = all_players_stock
                    .iter()
                    .filter(|(e, _)| *e != player_entity)
                    .max_by_key(|(_, stock)| stock.tokens_in_stock())
                    .map(|(e, _)| e);

                if let Some(b) = beneficiary {
                    let beneficiary_has_military = all_players_civ_cards
                        .get(b)
                        .map(|(_, c)| c.owns(&CivCardName::Military))
                        .unwrap_or(false);
                    if beneficiary_has_military {
                        state.apply_military_bonus();
                        info!("[CIVIL_WAR] Beneficiary has Military (+5 pts)");
                    }
                    info!(
                        "[CIVIL_WAR] Beneficiary {:?}; victim yields {} pts, beneficiary takes up to {} pts",
                        b, state.victim_selection_points, state.beneficiary_selection_points
                    );
                    state.beneficiary = Some(b);
                    state.phase = CivilWarPhase::VictimSelectsUnits;
                } else {
                    state.phase = CivilWarPhase::Complete;
                }
            }
            CivilWarPhase::VictimSelectsUnits => {
                if victim_is_human && cw_selection.acting_player.is_none() {
                    // First entry: set up human selection
                    let total_tokens: usize = victim_areas
                        .areas_and_population()
                        .into_iter()
                        .map(|(_, tokens)| tokens.len())
                        .sum();
                    let cities: Vec<Entity> = victim_cities.areas_and_cities.keys().cloned().collect();
                    cw_selection.populate_victim(
                        player_entity,
                        total_tokens,
                        cities,
                        state.victim_selection_points,
                    );
                    commands.entity(player_entity).insert(AwaitingHumanCalamitySelection);
                } else if victim_is_human && victim_awaiting {
                    // Still waiting for human UI
                } else if victim_is_human && !victim_awaiting && cw_selection.acting_player == Some(player_entity) {
                    // Human confirmed — read result
                    let (token_count, selected_city_areas) = cw_selection.take_result();
                    // Collect actual token entities (pick first N from areas)
                    let mut remaining = token_count;
                    for (_, tokens) in victim_areas.areas_and_population() {
                        for token in tokens {
                            if remaining == 0 { break; }
                            state.victim_selected_units.push(token);
                            remaining -= 1;
                        }
                        if remaining == 0 { break; }
                    }
                    for area in selected_city_areas {
                        state.victim_selected_cities.push(area);
                    }
                    let pts = state.victim_selected_units.len() + state.victim_selected_cities.len() * 5;
                    info!("[CIVIL_WAR] Human victim selected {} pts ({} tokens, {} cities)",
                        pts, state.victim_selected_units.len(), state.victim_selected_cities.len());
                    state.phase = CivilWarPhase::BeneficiarySelectsUnits;
                } else {
                    // AI victim: auto-select tokens then cities
                    let target = state.victim_selection_points;
                    let mut pts = 0usize;
                    'outer: for (_, tokens) in victim_areas.areas_and_population() {
                        for token in tokens {
                            if pts >= target { break 'outer; }
                            state.victim_selected_units.push(token);
                            pts += 1;
                        }
                    }
                    if pts < target {
                        for &area in victim_cities.areas_and_cities.keys() {
                            if pts >= target { break; }
                            state.victim_selected_cities.push(area);
                            pts += 5;
                        }
                    }
                    info!("[CIVIL_WAR] AI victim selected {} pts ({} tokens, {} cities)",
                        pts, state.victim_selected_units.len(), state.victim_selected_cities.len());
                    state.phase = CivilWarPhase::BeneficiarySelectsUnits;
                }
            }
            CivilWarPhase::BeneficiarySelectsUnits => {
                let Some(beneficiary) = state.beneficiary else {
                    state.phase = CivilWarPhase::TransferFaction;
                    continue;
                };

                let beneficiary_is_human = human_query.get(beneficiary).is_ok();
                let beneficiary_still_waiting = beneficiary_waiting.get(beneficiary).is_ok();

                if beneficiary_is_human && cw_selection.acting_player.is_none() {
                    // First entry: set up beneficiary human selection from victim's pool
                    let pool_tokens = state.victim_selected_units.len();
                    let pool_cities = state.victim_selected_cities.clone();
                    cw_selection.populate_beneficiary(
                        beneficiary,
                        pool_tokens,
                        pool_cities,
                        state.beneficiary_selection_points,
                    );
                    commands.entity(beneficiary).insert(AwaitingHumanCalamitySelection);
                } else if beneficiary_is_human && beneficiary_still_waiting {
                    // Still waiting for beneficiary UI
                } else if beneficiary_is_human && !beneficiary_still_waiting && cw_selection.acting_player == Some(beneficiary) {
                    // Beneficiary human confirmed
                    let (token_count, selected_city_areas) = cw_selection.take_result();
                    state.beneficiary_selected_units = state.victim_selected_units
                        .iter().copied().take(token_count).collect();
                    state.beneficiary_selected_cities = selected_city_areas;
                    let pts = state.beneficiary_selected_units.len() + state.beneficiary_selected_cities.len() * 5;
                    info!("[CIVIL_WAR] Human beneficiary takes {} pts", pts);
                    state.phase = CivilWarPhase::TransferFaction;
                } else {
                    // AI beneficiary: auto-select tokens then cities
                    let target = state.beneficiary_selection_points;
                    let mut pts = 0usize;
                    let mut take_tokens = Vec::new();
                    let mut take_cities = Vec::new();
                    for &token in &state.victim_selected_units {
                        if pts >= target { break; }
                        take_tokens.push(token);
                        pts += 1;
                    }
                    if pts < target {
                        for &area in &state.victim_selected_cities {
                            if pts >= target { break; }
                            take_cities.push(area);
                            pts += 5;
                        }
                    }
                    state.beneficiary_selected_units = take_tokens;
                    state.beneficiary_selected_cities = take_cities;
                    info!("[CIVIL_WAR] AI beneficiary takes {} pts", pts);
                    state.phase = CivilWarPhase::TransferFaction;
                }
            }
            CivilWarPhase::TransferFaction => {
                let Some(beneficiary) = state.beneficiary else {
                    state.phase = CivilWarPhase::Complete;
                    continue;
                };

                // Return non-transferred tokens from victim's selection back to stock
                for &token in state.victim_selected_units.iter()
                    .filter(|t| !state.beneficiary_selected_units.contains(t))
                {
                    commands.entity(token).insert(ReturnTokenToStock);
                }

                // Transfer tokens to beneficiary
                for &token in &state.beneficiary_selected_units {
                    commands.entity(token).insert(Token::new(beneficiary));
                }

                // Cities: transfer victim's selected cities to beneficiary
                for &area in &state.beneficiary_selected_cities {
                    commands.entity(area).insert(TransferCityTo(beneficiary));
                }

                // Return victim's non-taken cities via ReduceCity
                for &area in state.victim_selected_cities.iter()
                    .filter(|a| !state.beneficiary_selected_cities.contains(a))
                {
                    commands.entity(area).insert(ReduceCity);
                }

                info!("[CIVIL_WAR] Transfer complete");
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
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, awaiting_human) in
        player_query.iter_mut()
    {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::Treachery(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            TreacheryPhase::SelectCity => {
                if !is_human {
                    // AI: auto-select first city
                    let city_area = player_cities.areas_and_cities.keys().next().cloned();
                    if let Some(area) = city_area {
                        state.city_to_replace = Some(area);
                        state.beneficiary = resolution.context.traded_by;
                        state.phase = TreacheryPhase::ApplyEffects;
                    } else {
                        info!("[TREACHERY] No cities for player {:?}", player_entity);
                        state.phase = TreacheryPhase::Complete;
                    }
                } else if awaiting_human {
                    // Waiting for human UI
                } else if calamity_selection.player == Some(player_entity) {
                    // Human confirmed their selection
                    let selected = calamity_selection.take_selected_cities();
                    if let Some(&area) = selected.first() {
                        state.city_to_replace = Some(area);
                        state.beneficiary = resolution.context.traded_by;
                    }
                    state.phase = TreacheryPhase::ApplyEffects;
                } else {
                    // First time: set up human UI
                    let available: Vec<Entity> =
                        player_cities.areas_and_cities.keys().cloned().collect();
                    if available.is_empty() {
                        info!("[TREACHERY] No cities for human player {:?}", player_entity);
                        state.phase = TreacheryPhase::Complete;
                    } else {
                        calamity_selection.populate(player_entity, available, 1, "Treachery");
                        commands
                            .entity(player_entity)
                            .insert(AwaitingHumanCalamitySelection);
                    }
                }
            }
            TreacheryPhase::ApplyEffects => {
                if let Some(city_area) = state.city_to_replace {
                    if let Some(beneficiary) = state.beneficiary {
                        // Transfer city ownership to the player who traded the Treachery card (30.221)
                        commands
                            .entity(city_area)
                            .insert(TransferCityTo(beneficiary));
                    } else {
                        // Not traded – victim reduces own city, no one benefits (30.222)
                        commands.entity(city_area).insert(ReduceCity);
                    }
                }
                state.phase = TreacheryPhase::Complete;
            }
            TreacheryPhase::Complete => {
                finish_calamity(
                    &mut resolution,
                    &mut calamity_resolved,
                    &mut commands,
                    player_entity,
                    TradeCard::Treachery,
                );
            }
        }
    }
}

pub fn advance_piracy(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    all_players: Query<(Entity, &TokenStock, &PlayerCities), With<Player>>,
    area_query: Query<&BuiltCity>,
    sea_passage_query: Query<Has<SeaPassage>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, awaiting_human) in player_query.iter_mut() {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Piracy(ref mut state) = *resolving else { continue };

        match state.phase {
            PiracyPhase::DetermineBeneficiary => {
                let immune = resolution.context.traded_by;
                let beneficiary = all_players
                    .iter()
                    .filter(|(e, _, _)| *e != player_entity && Some(*e) != immune)
                    .max_by_key(|(_, stock, _)| stock.tokens_in_stock())
                    .map(|(e, _, _)| e);

                if let Some(b) = beneficiary {
                    state.beneficiary = Some(b);
                    state.phase = PiracyPhase::SelectCoastalCities;
                } else {
                    state.phase = PiracyPhase::Complete;
                }
            }
            PiracyPhase::SelectCoastalCities => {
                // Rule 30.911: beneficiary selects 2 coastal cities from primary victim.
                // (In our impl, we auto-select for the beneficiary and let the primary victim
                //  choose secondary targets in the next phase.)
                let coastal_cities: Vec<Entity> = player_cities.areas_and_cities.keys()
                    .filter(|&&area| sea_passage_query.get(area).unwrap_or(false))
                    .cloned()
                    .take(2)
                    .collect();
                let cities = if coastal_cities.is_empty() {
                    player_cities.areas_and_cities.keys().cloned().take(2).collect()
                } else {
                    coastal_cities
                };
                state.cities_to_replace = cities;
                state.phase = PiracyPhase::SelectSecondaryVictims;
            }
            PiracyPhase::SelectSecondaryVictims => {
                // Rule 30.912: primary victim chooses 2 other players to each lose 1 coastal city.
                let immune = resolution.context.traded_by;

                if is_human && calamity_selection.player.is_none() && !awaiting_human {
                    // Collect all coastal cities from eligible secondary players
                    let available: Vec<Entity> = all_players.iter()
                        .filter(|(e, _, cities)| {
                            *e != player_entity
                                && Some(*e) != immune
                                && !cities.areas_and_cities.is_empty()
                        })
                        .flat_map(|(_, _, cities)| {
                            let coastal: Vec<Entity> = cities.areas_and_cities.keys()
                                .filter(|&&area| sea_passage_query.get(area).unwrap_or(false))
                                .cloned()
                                .collect();
                            if coastal.is_empty() {
                                cities.areas_and_cities.keys().take(1).cloned().collect::<Vec<_>>()
                            } else {
                                coastal.into_iter().take(1).collect::<Vec<_>>()
                            }
                        })
                        .collect();

                    if available.len() < 2 {
                        // Not enough secondary victims, just take what we can
                        state.secondary_cities = available;
                        state.phase = PiracyPhase::ApplyEffects;
                    } else {
                        calamity_selection.populate(
                            player_entity,
                            available,
                            2,
                            "Piracy — Select 2 secondary victims",
                        );
                        commands.entity(player_entity).insert(AwaitingHumanCalamitySelection);
                    }
                } else if is_human && awaiting_human {
                    // Waiting for UI
                } else if is_human && !awaiting_human && calamity_selection.player == Some(player_entity) {
                    // Human confirmed
                    state.secondary_cities = calamity_selection.take_selected_cities();
                    state.phase = PiracyPhase::ApplyEffects;
                } else {
                    // AI: auto-select up to 1 coastal city from each of 2 other eligible players
                    let mut secondary: Vec<Entity> = Vec::new();
                    for (e, _, cities) in all_players.iter() {
                        if secondary.len() >= 2 { break; }
                        if e == player_entity || Some(e) == immune { continue; }
                        let coastal: Option<Entity> = cities.areas_and_cities.keys()
                            .find(|&&area| sea_passage_query.get(area).unwrap_or(false))
                            .or_else(|| cities.areas_and_cities.keys().next())
                            .copied();
                        if let Some(area) = coastal {
                            secondary.push(area);
                        }
                    }
                    state.secondary_cities = secondary;
                    state.phase = PiracyPhase::ApplyEffects;
                }
            }
            PiracyPhase::ApplyEffects => {
                if let Some(beneficiary) = state.beneficiary {
                    for &area in &state.cities_to_replace {
                        if area_query.get(area).is_ok() {
                            commands.entity(area).insert(TransferCityTo(beneficiary));
                        }
                    }
                    for &area in &state.secondary_cities {
                        if area_query.get(area).is_ok() {
                            commands.entity(area).insert(TransferCityTo(beneficiary));
                        }
                    }
                }
                info!(
                    "[PIRACY] Primary: {} cities, Secondary: {} cities",
                    state.cities_to_replace.len(), state.secondary_cities.len()
                );
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
    calamity_resolved.write(CalamityResolved {
        player: player_entity,
        calamity,
    });
    commands.entity(player_entity).remove::<ResolvingCalamity>();
    commands
        .entity(player_entity)
        .remove::<ActiveCalamityResolution>();
}

// ── Monotheism: post-calamity conversion (rule 32.94) ────────────────────────

/// After all calamities are resolved, Monotheism holders eliminate up to 2 enemy
/// tokens from areas adjacent to any of their occupied areas. Theology holders
/// are immune (rule 32.952). This system auto-processes for AI players; a human
/// UI hook can be added later. Once all holders are processed the system
/// transitions to CheckCitySupportAfterResolveCalamities.
pub fn apply_monotheism_conversions(
    mut commands: Commands,
    monotheism_holders: Query<(Entity, &PlayerAreas), With<NeedsMonotheismConversion>>,
    all_players_civ: Query<(Entity, &PlayerCivilizationCards)>,
    land_passage_query: Query<&LandPassage>,
    population_query: Query<&Population>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // Theology holders are immune to Monotheism (rule 32.952)
    let theology_immune: bevy::platform::collections::HashSet<Entity> = all_players_civ
        .iter()
        .filter(|(_, c)| c.owns(&CivCardName::Theology))
        .map(|(e, _)| e)
        .collect();

    let mut any_processed = false;
    for (holder_entity, holder_areas) in monotheism_holders.iter() {
        any_processed = true;

        // Find all enemy tokens in areas adjacent to any of this holder's areas
        let mut candidate_tokens: Vec<Entity> = Vec::new();
        'outer: for &held_area in holder_areas.areas().iter() {
            if let Ok(passages) = land_passage_query.get(held_area) {
                for &adj_area in &passages.to_areas {
                    if let Ok(pop) = population_query.get(adj_area) {
                        for (&enemy_player, tokens) in pop.player_tokens().iter() {
                            if enemy_player == holder_entity {
                                continue;
                            }
                            if theology_immune.contains(&enemy_player) {
                                continue;
                            }
                            for &token in tokens {
                                candidate_tokens.push(token);
                                if candidate_tokens.len() >= 2 {
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Mark the chosen tokens for return to stock (deferred, handled by existing system)
        for token in candidate_tokens {
            commands.entity(token).insert(ReturnTokenToStock);
            info!(
                "[MONOTHEISM] {:?} eliminates token {:?}",
                holder_entity, token
            );
        }

        commands
            .entity(holder_entity)
            .remove::<NeedsMonotheismConversion>();
    }

    if any_processed {
        info!("[MONOTHEISM] All conversions applied, transitioning to CheckCitySupport");
        next_state.set(GameActivity::CheckCitySupportAfterResolveCalamities);
    }
}

// ── Post-resolution cleanup ───────────────────────────────────────────────────

pub fn handle_calamity_resolved(
    mut events: MessageReader<CalamityResolved>,
    players_with_pending: Query<(Entity, &PendingCalamities), With<NeedsCalamityResolution>>,
    mut commands: Commands,
    names: Query<&Name>,
) {
    for event in events.read() {
        let player_name = names
            .get(event.player)
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        info!("[CALAMITIES] {} resolved {:?}", player_name, event.calamity);

        if let Ok((player_entity, pending)) = players_with_pending.get(event.player)
            && pending.is_empty()
        {
            commands
                .entity(player_entity)
                .remove::<NeedsCalamityResolution>();
            commands.entity(player_entity).remove::<PendingCalamities>();
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
        info!(
            "[CALAMITIES] Clearing all tokens from area {:?}",
            area_entity
        );
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

        if let Ok((mut city_stock, mut player_cities)) = city_stock_query.get_mut(built_city.player)
        {
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
    mut city_stock_query: Query<(
        &mut CityTokenStock,
        &mut PlayerCities,
        &mut TokenStock,
        &mut PlayerAreas,
    )>,
    mut move_tokens: MessageWriter<crate::civilization::events::MoveTokensFromStockToAreaCommand>,
) {
    for (area_entity, built_city, population) in areas_with_reduce.iter() {
        info!("[CALAMITIES] Reducing city in area {:?}", area_entity);

        if let Ok((mut city_stock, mut player_cities, _, _)) =
            city_stock_query.get_mut(built_city.player)
        {
            player_cities.remove_city_from_area(area_entity);
            city_stock.return_token_to_stock(built_city.city);

            move_tokens.write(
                crate::civilization::events::MoveTokensFromStockToAreaCommand {
                    player_entity: built_city.player,
                    area_entity,
                    number_of_tokens: population.max_population.min(6),
                },
            );
        }

        commands.entity(area_entity).remove::<BuiltCity>();
        commands.entity(area_entity).remove::<ReduceCity>();
    }
}

/// Marker: transfer the city in this area to `0` (a different player).
/// Used by Treachery, Civil War city transfer, and Piracy.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TransferCityTo(pub Entity);

/// System that processes `TransferCityTo`: removes the current owner's city and
/// builds a replacement city for the new owner (if they have a city token in stock).
pub fn transfer_city_to_new_owner(
    mut commands: Commands,
    areas_with_transfer: Query<(Entity, &BuiltCity, &TransferCityTo)>,
    mut player_data: Query<(&mut CityTokenStock, &mut PlayerCities)>,
) {
    let transfers: Vec<(Entity, Entity, Entity)> = areas_with_transfer
        .iter()
        .map(|(area, built_city, transfer)| (area, built_city.player, transfer.0))
        .collect();

    for (area_entity, victim_player, new_owner) in transfers {
        // Remove current BuiltCity and marker
        commands.entity(area_entity).remove::<BuiltCity>();
        commands.entity(area_entity).remove::<TransferCityTo>();

        // Use unsafe multi-get to borrow both entities mutably at once
        if victim_player != new_owner
            && let Ok([(mut v_stock, mut v_cities), (mut b_stock, mut b_cities)]) =
                player_data.get_many_mut([victim_player, new_owner])
        {
            if let Some(old_city) = v_cities.remove_city_from_area(area_entity) {
                v_stock.return_token_to_stock(old_city);
            }
            if let Some(new_city) = b_stock.get_token_from_stock() {
                b_cities.build_city_in_area(area_entity, new_city);
                commands
                    .entity(area_entity)
                    .insert(BuiltCity::new(new_owner, new_city));
                info!("[CALAMITIES] City transferred to {:?}", new_owner);
            } else {
                info!(
                    "[CALAMITIES] New owner {:?} has no city tokens; city lost",
                    new_owner
                );
            }
        }

        commands.entity(area_entity).insert(FixTokenPositions);
    }
}
