use bevy::prelude::*;
use rand::rng;
use rand::seq::SliceRandom;

use crate::GameActivity;
use crate::civilization::components::*;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::resolve_calamities::calamities::ResolvingCalamity;
use crate::civilization::concepts::resolve_calamities::calamities::barbarian_hordes::{
    barbarian_damage_score, BarbarianHordesPhase, BarbarianHordesState, MAX_CASCADE_ITERATIONS,
};
use crate::civilization::concepts::resolve_calamities::calamities::civil_disorder::{
    CivilDisorderPhase, CivilDisorderState,
};
use crate::civilization::concepts::resolve_calamities::calamities::civil_war::{
    CivilWarPhase, CivilWarState, FactionChoice,
};
use crate::civilization::concepts::resolve_calamities::calamities::epidemic::{
    allocate_removal_leaving_one_per_area, allocate_secondary_loss as allocate_epidemic_secondary_loss,
    EpidemicPhase, EpidemicState,
};
use crate::civilization::concepts::resolve_calamities::calamities::famine::{
    allocate_secondary_loss as allocate_famine_secondary_loss, FaminePhase, FamineState,
};
use crate::civilization::concepts::resolve_calamities::calamities::flood::{
    allocate_secondary_loss, FloodPhase, FloodState,
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
    AwaitingHumanCalamitySelection, AwaitingMonotheismSelection, CalamitySelectionState,
    CivilWarSelectionState, EpidemicSelectionState, FamineSelectionState, FloodSelectionState,
    MonotheismSelectionState,
};
use crate::civilization::concepts::conflict::{ConflictCounterResource, UnresolvedCityConflict, UnresolvedConflict};
use crate::civilization::functions::return_all_tokens_to_stock;
use crate::civilization::{CivCardName, PlayerTradeCards, TradeCard, TradeCardTrait};
use crate::loading::TextureAssets;
use crate::player::Player;
use crate::stupid_ai::IsHuman;

/// Rule 30.312: Grain cards locked by a Famine/Pottery reduction stop being
/// locked "the following turn" -- i.e. once the next round's Population
/// Expansion phase begins.
pub fn clear_grain_lock_for_new_turn(
    mut commands: Commands,
    locked: Query<Entity, With<GrainLockedForPurchase>>,
) {
    for player in &locked {
        commands.entity(player).remove::<GrainLockedForPurchase>();
    }
}

pub fn start_calamity_resolution(
    mut commands: Commands,
    players_with_calamities: Query<(Entity, &PlayerTradeCards), With<Player>>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    info!("[CALAMITIES] Starting calamity resolution phase");

    let mut any_calamities = false;

    for (player_entity, trade_cards) in players_with_calamities.iter() {
        let calamity_cards: Vec<TradeCard> = trade_cards.calamity_cards().iter().copied().collect();

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
        for (calamity, traded_by) in &pending.calamities {
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
                        .is_some_and(|c| c.owns(&CivCardName::Pottery));
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
                        // Rule 30.4124: Philosophy overrides the whole selection
                        // step "regardless of any other civilization cards" --
                        // so Music/Drama and Poetry/Democracy never apply
                        // alongside it.
                        if c.owns(&CivCardName::Philosophy) {
                            state = state.with_philosophy_override();
                        } else {
                            if c.owns(&CivCardName::Music) {
                                state.apply_music_bonus();
                            }
                            if c.owns(&CivCardName::DramaAndPoetry) {
                                state.apply_drama_poetry_bonus();
                            }
                            if c.owns(&CivCardName::Democracy) {
                                state.apply_democracy_bonus();
                            }
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
                        .is_some_and(|c| c.owns(&CivCardName::Mining));
                    let has_enlightenment = civ_cards
                        .is_some_and(|c| c.owns(&CivCardName::Enlightenment));
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
                        .is_some_and(|c| c.owns(&CivCardName::Engineering));
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
                    // Rule 30.52 never mentions Military as a modifier -- see
                    // BarbarianHordesState's doc comment.
                    commands.entity(*player_entity).insert((
                        ActiveCalamityResolution::new(context),
                        ResolvingCalamity::BarbarianHordes(BarbarianHordesState::new()),
                    ));
                }
                TradeCard::Epidemic => {
                    let has_medicine = civ_cards
                        .is_some_and(|c| c.owns(&CivCardName::Medicine));
                    let has_road_building = civ_cards
                        .is_some_and(|c| c.owns(&CivCardName::RoadBuilding));
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
            .map_or_else(|_| "Unknown".to_string(), std::string::ToString::to_string);

        info!("[VOLCANO/EARTHQUAKE] Resolving for {}", player_name);

        let has_engineering = player_civ_cards
            .get(primary_victim)
            .is_ok_and(|cards: &PlayerCivilizationCards| cards.owns(&CivCardName::Engineering));

        let player_cities_component = player_cities.get(primary_victim).ok();

        let volcano_result =
            find_best_volcano_eruption(primary_victim, &volcano_areas, &area_query);

        let state = if let Some((volcano_area, areas_to_clear)) = volcano_result {
            info!("[VOLCANO] Eruption at area {:?}", volcano_area);
            VolcanoEarthquakeState::as_volcano(volcano_area, areas_to_clear)
        } else {
            info!("[EARTHQUAKE] No volcano areas with cities, triggering earthquake");

            if let Some(cities) = player_cities_component {
                let city_areas: Vec<Entity> = cities.areas_and_cities.keys().copied().collect();

                if city_areas.is_empty() {
                    VolcanoEarthquakeState::new()
                } else {
                    let city_to_destroy = city_areas[0];
                    let city_to_reduce =
                        find_adjacent_city_to_reduce(city_to_destroy, primary_victim, &area_query, &player_civ_cards);

                    info!(
                        "[EARTHQUAKE] City to destroy: {:?}, city to reduce: {:?}, has_engineering: {}",
                        city_to_destroy, city_to_reduce, has_engineering
                    );

                    VolcanoEarthquakeState::as_earthquake(
                        city_to_destroy,
                        city_to_reduce,
                        has_engineering,
                    )
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

            for adjacent_area in &land_passage.to_areas {
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

/// Rule 30.612: find an adjacent enemy city to reduce as the earthquake secondary victim.
/// Players holding Engineering are immune (rule 30.612 modifier — cannot be secondary).
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
    player_civ_cards: &Query<&PlayerCivilizationCards>,
) -> Option<Entity> {
    if let Ok((_, _, _, _, land_passage)) = area_query.get(city_area) {
        for adjacent_area in &land_passage.to_areas {
            if let Ok((_, _, Some(adj_city), _, _)) = area_query.get(*adjacent_area)
                && adj_city.player != primary_victim
            {
                // Engineering holders are immune to being Earthquake secondary victims.
                let secondary_has_engineering = player_civ_cards
                    .get(adj_city.player)
                    .is_ok_and(|c| c.owns(&CivCardName::Engineering));
                if secondary_has_engineering {
                    info!(
                        "[EARTHQUAKE] Player {:?} holds Engineering — immune to secondary",
                        adj_city.player
                    );
                    continue;
                }
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
    for (player_entity, mut resolution, resolving) in &mut players_resolving {
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
        areas_to_clear.extend(land_passage.to_areas.iter().copied());
    }

    for area in &areas_to_clear {
        if let Ok((area_entity, _population, built_city, _, _)) = area_query.get(*area) {
            commands.entity(area_entity).insert(FixTokenPositions);

            if let Some(city) = built_city {
                commands.entity(area_entity).remove::<BuiltCity>();
                commands.entity(city.city).insert(ReturnCityToStock);
            }
        }
    }

    for area in &areas_to_clear {
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
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    flood_plains: Query<Entity, With<FloodPlain>>,
    area_query: Query<(Option<&BuiltCity>, &LandPassage)>,
    mut populations: Query<&mut Population>,
    sea_passage_query: Query<Has<SeaPassage>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut flood_selection: ResMut<FloodSelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, is_awaiting) in &mut player_query {
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
                        .map_or(0, |pop| pop.population_for_player(player_entity));
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
                // Rule 30.511: the primary victim loses a MAXIMUM of `primary_max_loss`
                // unit points (17, or 7 with Engineering) from THIS flood plain only —
                // not an unbounded wipe, and not areas adjacent to it (that behavior
                // belongs to Volcano/Earthquake, rule 30.211, not Flood).
                if let Some(fp_area) = state.flood_plain_area {
                    let mut cap = state.primary_max_loss.max(0) as usize;
                    if let Ok((built_city, _)) = area_query.get(fp_area) {
                        let victim_owns_city = built_city.is_some_and(|c| c.player == player_entity);
                        if victim_owns_city && cap > 0 {
                            let city_cost = cap.min(5);
                            if state.has_engineering {
                                commands.entity(fp_area).insert(ReduceCity);
                            } else {
                                commands.entity(fp_area).insert(DestroyCity);
                            }
                            cap -= city_cost;
                        }
                    }
                    if cap > 0
                        && let Ok(mut pop) = populations.get_mut(fp_area)
                    {
                        let available = pop.population_for_player(player_entity);
                        let to_remove = cap.min(available);
                        if to_remove > 0
                            && let Some(removed) = pop.remove_tokens_from_area(&player_entity, to_remove)
                        {
                            for token in removed {
                                commands.entity(token).insert(ReturnTokenToStock);
                            }
                        }
                    }
                }
                state.phase = FloodPhase::ApplySecondaryLoss;
            }
            FloodPhase::ApplySecondaryLoss => {
                // Rule 30.512: secondary victims on the same flood plain collectively
                // lose 10 pts. The primary victim divides that loss among them
                // (allocate_secondary_loss); a human primary victim gets an
                // interactive choice via FloodSelectionState (mirrors the
                // Civil War selection pattern), AI falls back to an even
                // split. If combined secondary availability is <=10 there's
                // no decision to make at all -- everyone automatically loses
                // everything, so the UI is skipped entirely in that case.
                let Some(fp_area) = state.flood_plain_area else {
                    state.phase = FloodPhase::Complete;
                    continue;
                };
                let Ok(pop) = populations.get(fp_area) else {
                    state.phase = FloodPhase::Complete;
                    continue;
                };
                let secondary_players: Vec<(Entity, usize)> = pop
                    .player_tokens()
                    .keys()
                    .filter(|&&e| e != player_entity)
                    .map(|&e| (e, pop.population_for_player(e)))
                    .collect();
                let n = secondary_players.len();
                if n == 0 {
                    state.phase = FloodPhase::Complete;
                    continue;
                }

                let total_loss = 10usize;
                let combined_available: usize = secondary_players.iter().map(|&(_, a)| a).sum();
                let needs_choice = combined_available > total_loss;

                let allocation = if !needs_choice {
                    allocate_secondary_loss(&secondary_players, total_loss, None)
                } else if is_human {
                    if flood_selection.acting_player.is_none() {
                        flood_selection.populate(player_entity, secondary_players.clone(), total_loss);
                        commands.entity(player_entity).insert(AwaitingHumanCalamitySelection);
                        continue; // wait for the human this frame
                    } else if is_awaiting {
                        continue; // still waiting on the UI
                    } else if flood_selection.acting_player == Some(player_entity) {
                        let choice = flood_selection.take_result();
                        allocate_secondary_loss(&secondary_players, total_loss, Some(&choice))
                    } else {
                        continue; // selection resource is owned by a different player right now
                    }
                } else {
                    allocate_secondary_loss(&secondary_players, total_loss, None)
                };

                if let Ok(mut pop) = populations.get_mut(fp_area) {
                    for (sec, to_remove) in &allocation {
                        if *to_remove == 0 {
                            continue;
                        }
                        if let Some(removed) = pop.remove_tokens_from_area(sec, *to_remove) {
                            for token in removed {
                                commands.entity(token).insert(ReturnTokenToStock);
                            }
                        }
                    }
                }
                info!("[FLOOD] Applied secondary loss of {} pts ({} secondary victims)", total_loss, n);
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
                    .copied();
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

/// Epidemic-specific variant of `remove_unit_points`: rule 30.612 requires
/// leaving at least one token in each affected area, unlike every other
/// unit-point-loss calamity. Uses `allocate_removal_leaving_one_per_area` to
/// compute the per-area cap before removing anything.
fn remove_unit_points_leaving_one_per_area(
    player: Entity,
    points: i32,
    player_areas: &PlayerAreas,
    populations: &mut Query<&mut Population>,
    commands: &mut Commands,
) {
    if points <= 0 {
        return;
    }
    let areas_and_counts: Vec<(Entity, usize)> =
        player_areas.areas_and_population_count().into_iter().collect();
    let counts: Vec<usize> = areas_and_counts.iter().map(|(_, c)| *c).collect();
    let allocation = allocate_removal_leaving_one_per_area(&counts, points as usize);

    for ((area, _), to_remove) in areas_and_counts.iter().zip(allocation) {
        if to_remove == 0 {
            continue;
        }
        if let Ok(mut population) = populations.get_mut(*area)
            && let Some(removed) = population.remove_tokens_from_area(&player, to_remove)
        {
            for token in removed {
                commands.entity(token).insert(ReturnTokenToStock);
            }
        }
    }
}

/// Rule 30.612: "Cities eliminated by Epidemic are replaced by at least one
/// token; thus cities account for a maximum of 4 unit points when
/// calculating Epidemic losses." Unlike Flood/Volcano, the rule text gives
/// this number directly and never mentions Engineering, so Engineering has
/// no effect on Epidemic city elimination -- always a plain `DestroyCity`,
/// never `ReduceCity`.
///
/// A city's cost is deducted from the loss budget before any tokens are
/// removed (mirroring Flood's `ApplyPrimaryLoss` city-then-tokens order,
/// the closest existing precedent for a budget covering both). If a player
/// has multiple cities and the remaining budget covers more than one, they
/// are eliminated in `player_cities.areas_and_cities` iteration order until
/// the budget runs out or the player has no cities left -- deterministic
/// auto-selection, not an interactive choice (matching how other "which of
/// several targets" specifics are auto-resolved elsewhere pending UI work).
///
/// Returns the remaining budget after city costs, to spend on tokens via
/// `remove_unit_points_leaving_one_per_area`.
fn spend_epidemic_budget_on_cities(
    player_cities: &PlayerCities,
    points: i32,
    commands: &mut Commands,
) -> i32 {
    const CITY_COST: i32 = 4;
    let mut remaining = points;
    for &area in player_cities.areas_and_cities.keys() {
        if remaining <= 0 {
            break;
        }
        let city_cost = remaining.min(CITY_COST);
        commands.entity(area).insert(DestroyCity);
        remaining -= city_cost;
    }
    remaining
}

/// Epidemic loss resolution: city cost (rule 30.612, capped at 4/city) is
/// deducted first, then whatever remains is removed from tokens via
/// `remove_unit_points_leaving_one_per_area`. Used for both primary and
/// secondary Epidemic victims -- 30.612 doesn't distinguish between them.
fn remove_epidemic_loss(
    player: Entity,
    points: i32,
    player_areas: &PlayerAreas,
    player_cities: &PlayerCities,
    populations: &mut Query<&mut Population>,
    commands: &mut Commands,
) {
    let remaining = spend_epidemic_budget_on_cities(player_cities, points, commands);
    remove_unit_points_leaving_one_per_area(player, remaining, player_areas, populations, commands);
}

pub fn advance_famine(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut ResolvingCalamity,
        &mut ActiveCalamityResolution,
        &PlayerAreas,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    mut populations: Query<&mut Population>,
    all_players: Query<(Entity, &PlayerAreas), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut famine_selection: ResMut<FamineSelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas, is_human, is_awaiting) in &mut player_query {
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
                if state.grain_cards_used > 0 {
                    // Rule 30.312: these Grain cards are placed face up and locked
                    // until the following turn; cleared OnEnter(PopulationExpansion).
                    commands
                        .entity(player_entity)
                        .insert(GrainLockedForPurchase(state.grain_cards_used));
                    info!(
                        "[FAMINE] {} Grain card(s) placed face up, locked until next turn (30.312)",
                        state.grain_cards_used
                    );
                }
                info!("[FAMINE] Primary loss of {} applied", loss);
                state.phase = FaminePhase::SelectSecondaryVictims;
            }
            FaminePhase::SelectSecondaryVictims => {
                // Rule 30.311: secondary victims (players sharing an area with
                // the primary victim) collectively lose 20 pts, no more than 8
                // from any one of them. The primary victim divides that loss
                // among them (allocate_famine_secondary_loss); a human primary
                // victim gets an interactive choice via FamineSelectionState
                // (mirrors Flood's rule-30.512 pattern), AI falls back to an
                // even split. If combined secondary availability is <=20
                // there's no decision to make at all -- everyone automatically
                // loses everything, so the UI is skipped entirely in that case.
                let primary_areas: bevy::platform::collections::HashSet<Entity> =
                    player_areas.areas().iter().copied().collect();

                let max_per_player = state.max_per_secondary.max(0) as usize;
                let secondary_players: Vec<(Entity, usize)> = all_players
                    .iter()
                    .filter(|(e, areas)| {
                        *e != player_entity
                            && areas.areas().iter().any(|a| primary_areas.contains(a))
                    })
                    .map(|(e, areas)| {
                        let total_pop: usize =
                            areas.areas_and_population_count().values().sum();
                        (e, total_pop.min(max_per_player))
                    })
                    .collect();

                if secondary_players.is_empty() {
                    state.secondary_allocations.clear();
                    state.phase = FaminePhase::ApplySecondaryLosses;
                    continue;
                }

                let total_loss = state.secondary_total.max(0) as usize;
                let combined_available: usize =
                    secondary_players.iter().map(|&(_, a)| a).sum();
                let needs_choice = combined_available > total_loss;

                let allocation = if !needs_choice {
                    allocate_famine_secondary_loss(&secondary_players, total_loss, None)
                } else if is_human {
                    if famine_selection.acting_player.is_none() {
                        famine_selection.populate(player_entity, secondary_players.clone(), total_loss);
                        commands.entity(player_entity).insert(AwaitingHumanCalamitySelection);
                        continue; // wait for the human this frame
                    } else if is_awaiting {
                        continue; // still waiting on the UI
                    } else if famine_selection.acting_player == Some(player_entity) {
                        let choice = famine_selection.take_result();
                        allocate_famine_secondary_loss(&secondary_players, total_loss, Some(&choice))
                    } else {
                        continue; // selection resource is owned by a different player right now
                    }
                } else {
                    allocate_famine_secondary_loss(&secondary_players, total_loss, None)
                };

                state.secondary_allocations =
                    allocation.into_iter().map(|(e, p)| (e, p as i32)).collect();
                state.phase = FaminePhase::ApplySecondaryLosses;
            }
            FaminePhase::ApplySecondaryLosses => {
                // Secondary victims: players sharing areas with primary victim lose up to 20 pts
                // total, max 8 per player (rule 30.311), per the allocation computed in
                // SelectSecondaryVictims (human choice, or auto-distributed for AI).
                for (secondary_entity, loss) in state.secondary_allocations.clone() {
                    if loss <= 0 {
                        continue;
                    }
                    if let Ok((_, sec_areas)) = all_players.get(secondary_entity) {
                        remove_unit_points(
                            secondary_entity,
                            loss,
                            sec_areas,
                            &mut populations,
                            &mut commands,
                        );
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
            // Unreachable: ComputeLosses applies the primary loss directly and
            // transitions straight to SelectSecondaryVictims. Kept in the enum
            // as a documented phase name (mirrors Flood's phase list).
            FaminePhase::ApplyPrimaryLoss => {}
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
    start_areas: Query<(Entity, &StartArea)>,
    area_passages: Query<(Option<&LandPassage>, Option<&SeaPassage>, Has<OpenSea>)>,
    area_transforms: Query<&Transform, With<GameArea>>,
    city_query: Query<&BuiltCity>,
    conflict_marker_query: Query<(Has<UnresolvedConflict>, Has<UnresolvedCityConflict>)>,
    mut conflict_counter: ResMut<ConflictCounterResource>,
    mut populations: Query<&mut Population>,
    // Optional so headless/test worlds without the asset-loading resource
    // (which has ~20 unrelated Handle<Image> fields, tedious to construct
    // just to satisfy this) can still run the real placement/conflict/
    // cascade logic; only the cosmetic Sprite is skipped without it.
    textures: Option<Res<TextureAssets>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
) {
    for (player_entity, mut resolving, mut resolution, faction) in &mut player_query {
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

                // Rule 30.5211: place in the victim's start area causing the
                // greatest damage (30.5234's metric, reused here so placement
                // and every later cascade step use one consistent rule). If
                // the victim has no units (and no city) in any start area,
                // this naturally falls back to an empty start area, since
                // every start area still scores >=0 and ties keep the first
                // one seen.
                let victim_faction = faction.faction;
                let mut best_area: Option<Entity> = None;
                let mut best_score = 0usize;

                for (area_entity, start_area) in start_areas.iter() {
                    if start_area.faction != victim_faction { continue; }
                    let victim_tokens = populations.get(area_entity)
                        .map_or(0, |pop| pop.population_for_player(player_entity));
                    let has_city = city_query.get(area_entity).is_ok_and(|c| c.player == player_entity);
                    let score = barbarian_damage_score(victim_tokens, has_city);
                    if best_area.is_none() || score > best_score {
                        best_score = score;
                        best_area = Some(area_entity);
                    }
                }

                info!(
                    "[BARBARIAN_HORDES] {:?} landing in {:?} (damage score {})",
                    victim_faction, best_area, best_score
                );
                state.landing_area = best_area;
                state.phase = if best_area.is_some() {
                    BarbarianHordesPhase::PlaceBarbarians
                } else {
                    // Degenerate case: victim faction has no start areas at all.
                    BarbarianHordesPhase::Complete
                };
            }
            BarbarianHordesPhase::PlaceBarbarians => {
                // Rule 30.5211: fifteen tokens belonging to an unused nation.
                // The owner entity has no Player/Treasury/PlayerTradeCards/
                // PlayerCities/TokenStock/PlayerAreas -- see the state
                // struct's doc comment for why that's the point.
                const TOKEN_COUNT: usize = 15;
                const COLS: usize = 5;
                const SPACING: f32 = 10.0;

                let Some(landing) = state.landing_area else {
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                };

                let barbarian_entity = commands.spawn(Name::new("Barbarian Horde")).id();
                state.barbarian_entity = Some(barbarian_entity);

                let base = area_transforms.get(landing).map(|t| t.translation).unwrap_or_default();

                if let Ok(mut pop) = populations.get_mut(landing) {
                    for i in 0..TOKEN_COUNT {
                        let col = (i % COLS) as f32;
                        let row = (i / COLS) as f32;
                        let offset = bevy::math::Vec3::new(
                            col * SPACING - SPACING * 2.0,
                            row * -SPACING + SPACING,
                            3.0, // above player tokens (z = 0) and ships (z = 2)
                        );
                        let mut entity_commands = commands.spawn((
                            BarbarianToken,
                            Name::new("Barbarian"),
                            Transform::from_scale(bevy::math::Vec3::splat(0.3))
                                .with_translation(base + offset),
                        ));
                        if let Some(textures) = &textures {
                            entity_commands.insert(Sprite {
                                image: textures.dot.clone(),
                                color: bevy::prelude::Color::srgb(0.6, 0.1, 0.05),
                                ..Default::default()
                            });
                        }
                        let token = entity_commands.id();
                        pop.add_token_to_area(barbarian_entity, token);
                        state.all_tokens.push(token);
                    }
                }

                info!(
                    "[BARBARIAN_HORDES] Placed {} barbarian tokens at {:?} (owner {:?})",
                    TOKEN_COUNT, landing, barbarian_entity
                );
                state.phase = BarbarianHordesPhase::EnterArea;
            }
            BarbarianHordesPhase::EnterArea => {
                let Some(area) = state.landing_area else {
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                };
                if !state.visited_areas.contains(&area) {
                    state.visited_areas.push(area);
                }

                let has_city = city_query.get(area).is_ok();
                let is_conflict_zone = populations.get(area)
                    .is_ok_and(|p| p.is_conflict_zone(has_city));

                if is_conflict_zone {
                    // Rule 30.5212/30.5231: real conflict resolution, reusing
                    // this game's existing machinery rather than bespoke
                    // combat math. The observers that resolve these markers
                    // decrement the shared ConflictCounterResource and, if it
                    // reaches 0, transition GameActivity -- which must NEVER
                    // happen here (we're mid-calamity-resolution, not the
                    // Conflict phase). Padding the counter to 2 before
                    // inserting guarantees the post-resolution decrement
                    // lands on 1, never 0. The real Conflict phase always
                    // resets this counter to 0 itself before it starts
                    // (find_conflict_zones), so leaving it nonzero here is
                    // harmless.
                    conflict_counter.0 = 2;
                    if has_city {
                        commands.entity(area).insert(UnresolvedCityConflict);
                    } else {
                        commands.entity(area).insert(UnresolvedConflict);
                    }
                    state.phase = BarbarianHordesPhase::AwaitingConflict;
                } else {
                    state.phase = BarbarianHordesPhase::CheckSurplus;
                }
            }
            BarbarianHordesPhase::AwaitingConflict => {
                let Some(area) = state.landing_area else {
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                };
                let still_pending = conflict_marker_query.get(area)
                    .is_ok_and(|(a, b)| a || b);
                if still_pending {
                    continue; // Commands haven't flushed / observer hasn't run yet -- poll again next tick.
                }
                state.phase = BarbarianHordesPhase::CheckSurplus;
            }
            BarbarianHordesPhase::CheckSurplus => {
                let Some(area) = state.landing_area else {
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                };
                let Some(barbarian_entity) = state.barbarian_entity else {
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                };

                // Rule 30.5232: surplus_count() is exactly "population beyond
                // the area's max_population" -- and since normal conflict
                // resolution already reduces total population to at most
                // max_population whenever more than one owner remains, a
                // nonzero surplus here can only mean the Barbarians are the
                // sole remaining owner (or landed on an empty/undefended area
                // with no conflict at all), matching 30.5231's "surplus
                // Barbarians (beyond the area's population limit)".
                let surplus = populations.get(area).map_or(0, Population::surplus_count);

                if surplus == 0 || state.cascade_iterations >= MAX_CASCADE_ITERATIONS {
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                }

                // Rule 30.5231/30.5234: adjacent area (land or coastal sea
                // hop; never open sea, rule 30.5233) causing the greatest
                // damage to the primary victim. Ties keep the first
                // candidate seen (30.525's interactive tie-break -- the
                // trading player, or highest-stock player, choosing among
                // ties -- is out of scope for this pass; documented in
                // docs/outline.md).
                let (land, sea, _) = area_passages.get(area).unwrap_or((None, None, false));
                let mut candidates: Vec<Entity> = Vec::new();
                if let Some(lp) = land { candidates.extend(lp.to_areas.iter().copied()); }
                if let Some(sp) = sea { candidates.extend(sp.to_areas.iter().copied()); }

                let mut best: Option<(Entity, usize)> = None;
                for candidate in candidates {
                    let is_open_sea = area_passages.get(candidate).is_ok_and(|(_, _, open)| open);
                    if is_open_sea { continue; } // rule 30.5233
                    let has_city = city_query.get(candidate).is_ok_and(|c| c.player == player_entity);
                    let victim_tokens = populations.get(candidate)
                        .map_or(0, |p| p.population_for_player(player_entity));
                    let score = barbarian_damage_score(victim_tokens, has_city);
                    if best.is_none_or(|(_, b)| score > b) {
                        best = Some((candidate, score));
                    }
                }

                let Some((next_area, _)) = best else {
                    // Dead end: nowhere adjacent to send the surplus. No
                    // rules text covers this edge case on a real board;
                    // stopping here is the only sound option.
                    info!(
                        "[BARBARIAN_HORDES] {} surplus barbarian(s) have nowhere to advance from {:?} -- stopping",
                        surplus, area
                    );
                    state.phase = BarbarianHordesPhase::Complete;
                    continue;
                };

                if let Ok(mut pop) = populations.get_mut(area)
                    && let Some(moving) = pop.remove_tokens_from_area(&barbarian_entity, surplus)
                    && let Ok(mut next_pop) = populations.get_mut(next_area)
                {
                    for token in moving {
                        next_pop.add_token_to_area(barbarian_entity, token);
                    }
                }

                info!(
                    "[BARBARIAN_HORDES] {} surplus barbarian(s) advance from {:?} to {:?}",
                    surplus, area, next_area
                );
                state.landing_area = Some(next_area);
                state.cascade_iterations += 1;
                state.phase = BarbarianHordesPhase::EnterArea;
            }
            BarbarianHordesPhase::Complete => {
                // KNOWN SIMPLIFICATION (see the state struct's doc comment):
                // rule 30.5235 says surviving Barbarians remain on the board
                // until eliminated. This despawns everyone, winners included,
                // rather than persisting a Barbarian nation indefinitely.
                if let Some(barbarian_entity) = state.barbarian_entity {
                    for area in state.visited_areas.drain(..) {
                        if let Ok(mut pop) = populations.get_mut(area) {
                            pop.remove_all_tokens_for_player(&barbarian_entity);
                        }
                    }
                    commands.entity(barbarian_entity).despawn();
                }
                for token in state.all_tokens.drain(..) {
                    commands.entity(token).despawn();
                }
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
        &PlayerCities,
        Has<IsHuman>,
        Has<AwaitingHumanCalamitySelection>,
    )>,
    mut populations: Query<&mut Population>,
    all_players: Query<(Entity, &PlayerAreas, &PlayerCities, Option<&PlayerCivilizationCards>), With<Player>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut epidemic_selection: ResMut<EpidemicSelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_areas, player_cities, is_human, is_awaiting) in
        &mut player_query
    {
        if resolution.phase == CalamityPhase::Resolved {
            continue;
        }
        let ResolvingCalamity::Epidemic(ref mut state) = *resolving else {
            continue;
        };

        match state.phase {
            EpidemicPhase::ComputeEffects => {
                let loss = state.primary_loss;
                remove_epidemic_loss(
                    player_entity,
                    loss,
                    player_areas,
                    player_cities,
                    &mut populations,
                    &mut commands,
                );
                state.phase = EpidemicPhase::ApplySecondaryLosses;
            }
            EpidemicPhase::ApplySecondaryLosses => {
                // Rule 30.611: secondary victims sharing an area with the primary
                // victim collectively lose 25 pts, max 10 per player (5 with
                // Medicine, rule 30.613). Immune player (trader) is exempt. A
                // human primary victim divides the loss via EpidemicSelectionState
                // (mirrors Flood's 30.512 pattern); AI falls back to a greedy
                // fill in encounter order. If combined secondary caps are <=25
                // there's no decision to make at all -- everyone automatically
                // loses up to their own cap, so the UI is skipped entirely.
                let primary_areas: bevy::platform::collections::HashSet<Entity> =
                    player_areas.areas().iter().copied().collect();

                let secondary_total = state.secondary_loss.max(0) as usize;

                let secondary_players: Vec<(Entity, usize)> = all_players
                    .iter()
                    .filter(|(e, areas, _, _)| {
                        *e != player_entity
                            && Some(*e) != state.immune_player
                            && areas.areas().iter().any(|a| primary_areas.contains(a))
                    })
                    .map(|(e, _, _, cards)| {
                        let has_medicine = cards.is_some_and(|c| c.owns(&CivCardName::Medicine));
                        let cap = if has_medicine { 5 } else { 10 };
                        (e, cap)
                    })
                    .collect();

                if secondary_players.is_empty() {
                    state.phase = EpidemicPhase::Complete;
                    continue;
                }

                let combined_available: usize = secondary_players.iter().map(|&(_, a)| a).sum();
                let needs_choice = combined_available > secondary_total;

                let allocation = if !needs_choice {
                    allocate_epidemic_secondary_loss(&secondary_players, secondary_total, None)
                } else if is_human {
                    if epidemic_selection.acting_player.is_none() {
                        epidemic_selection.populate(player_entity, secondary_players.clone(), secondary_total);
                        commands.entity(player_entity).insert(AwaitingHumanCalamitySelection);
                        continue; // wait for the human this frame
                    } else if is_awaiting {
                        continue; // still waiting on the UI
                    } else if epidemic_selection.acting_player == Some(player_entity) {
                        let choice = epidemic_selection.take_result();
                        allocate_epidemic_secondary_loss(&secondary_players, secondary_total, Some(&choice))
                    } else {
                        continue; // selection resource is owned by a different player right now
                    }
                } else {
                    allocate_epidemic_secondary_loss(&secondary_players, secondary_total, None)
                };

                for (secondary_entity, to_remove) in &allocation {
                    if *to_remove == 0 {
                        continue;
                    }
                    if let Ok((_, sec_areas, sec_cities, _)) = all_players.get(*secondary_entity) {
                        remove_epidemic_loss(
                            *secondary_entity,
                            *to_remove as i32,
                            sec_areas,
                            sec_cities,
                            &mut populations,
                            &mut commands,
                        );
                        info!(
                            "[EPIDEMIC] Secondary player {:?} loses {} pts",
                            secondary_entity, to_remove
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
            EpidemicPhase::ApplyPrimaryLoss => {}
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
        &mut player_query
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
                        player_cities.areas_and_cities.keys().copied().collect();
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
                        .copied()
                        .take(state.cities_to_reduce)
                        .collect();
                    for area in &areas {
                        state.select_city(*area);
                    }
                    state.phase = IconoclasmHeresyPhase::ApplySecondaryLosses;
                }
            }
            IconoclasmHeresyPhase::SelectCities if !awaiting_human => {
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
                            .is_some_and(|c| c.owns(&CivCardName::Theology));
                        if has_theology {
                            return None;
                        } // Theology immune (30.819)
                        let has_philosophy = cards
                            .is_some_and(|c| c.owns(&CivCardName::Philosophy));
                        let city_areas: Vec<Entity> =
                            cities.areas_and_cities.keys().copied().collect();
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
        &mut player_query
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
                        player_cities.areas_and_cities.keys().copied().collect();
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
                        .copied()
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
    ) in &mut player_query
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
                        player_cities.areas_and_cities.keys().copied().collect();
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
                        .copied()
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
        &mut player_query
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
                        player_cities.areas_and_cities.keys().copied().collect();
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
                        .copied()
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

/// Rule 30.413/30.414: once both the victim's pick (30.4121/30.4122) and the
/// beneficiary's top-up (30.4123) are final, the first faction is complete.
/// Derives the second faction as the victim's whole board minus the first
/// faction, then applies the Military penalty (30.414) to both factions
/// before handing off to the victim's 30.415 choice.
fn finalize_first_faction(
    state: &mut CivilWarState,
    victim_areas: &PlayerAreas,
    victim_cities: &PlayerCities,
) {
    let full_units: Vec<Entity> = victim_areas
        .areas_and_population()
        .into_iter()
        .flat_map(|(_, tokens)| tokens.into_iter())
        .collect();
    let full_cities: Vec<Entity> = victim_cities.areas_and_cities.keys().copied().collect();
    state.compute_second_faction(full_units, full_cities);
    state.apply_military_penalty_to_first_faction();
    state.apply_military_penalty_to_second_faction();
    state.phase = CivilWarPhase::VictimChoosesFaction;
}

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
        &mut player_query
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
                    // Rule 30.414: Military removes 5 unit points from each
                    // faction "to reflect increased destructiveness" -- applies
                    // if either participant holds it, not just the beneficiary.
                    let victim_has_military = all_players_civ_cards
                        .get(player_entity)
                        .is_ok_and(|(_, c)| c.owns(&CivCardName::Military));
                    let beneficiary_has_military = all_players_civ_cards
                        .get(b)
                        .is_ok_and(|(_, c)| c.owns(&CivCardName::Military));
                    if victim_has_military || beneficiary_has_military {
                        state.military_penalty = true;
                        info!("[CIVIL_WAR] Military held by a participant -- 5 pts will be removed from each faction (30.414)");
                    }
                    info!(
                        "[CIVIL_WAR] Beneficiary {:?}; victim yields {} pts, beneficiary takes up to {} pts",
                        b, state.victim_selection_points, state.beneficiary_selection_points
                    );
                    state.beneficiary = Some(b);
                    state.phase = if state.philosophy_override {
                        CivilWarPhase::BeneficiarySelectsUnits
                    } else {
                        CivilWarPhase::VictimSelectsUnits
                    };
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
                    let cities: Vec<Entity> = victim_cities.areas_and_cities.keys().copied().collect();
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

                // Rule 30.4123: the beneficiary's top-up is drawn from what's
                // left of the victim's board (full board minus the victim's
                // own pick), not a subset of the victim's pick. Under
                // Philosophy (30.4124) the victim never selects anything, so
                // `victim_selected_*` is empty and this pool is naturally the
                // victim's whole board.
                let pool_tokens: Vec<Entity> = victim_areas
                    .areas_and_population()
                    .into_iter()
                    .flat_map(|(_, tokens)| tokens.into_iter())
                    .filter(|t| !state.victim_selected_units.contains(t))
                    .collect();
                let pool_cities: Vec<Entity> = victim_cities
                    .areas_and_cities
                    .keys()
                    .copied()
                    .filter(|c| !state.victim_selected_cities.contains(c))
                    .collect();

                if beneficiary_is_human && cw_selection.acting_player.is_none() {
                    // First entry: set up beneficiary human selection from the pool
                    cw_selection.populate_beneficiary(
                        beneficiary,
                        pool_tokens.len(),
                        pool_cities.clone(),
                        state.beneficiary_selection_points,
                    );
                    commands.entity(beneficiary).insert(AwaitingHumanCalamitySelection);
                } else if beneficiary_is_human && beneficiary_still_waiting {
                    // Still waiting for beneficiary UI
                } else if beneficiary_is_human && !beneficiary_still_waiting && cw_selection.acting_player == Some(beneficiary) {
                    // Beneficiary human confirmed
                    let (token_count, selected_city_areas) = cw_selection.take_result();
                    state.beneficiary_selected_units = pool_tokens
                        .iter().copied().take(token_count).collect();
                    state.beneficiary_selected_cities = selected_city_areas;
                    let pts = state.beneficiary_selected_units.len() + state.beneficiary_selected_cities.len() * 5;
                    info!("[CIVIL_WAR] Human beneficiary takes {} pts", pts);
                    finalize_first_faction(state, victim_areas, victim_cities);
                } else {
                    // AI beneficiary: auto-select tokens then cities
                    let target = state.beneficiary_selection_points;
                    let mut pts = 0usize;
                    let mut take_tokens = Vec::new();
                    let mut take_cities = Vec::new();
                    for &token in &pool_tokens {
                        if pts >= target { break; }
                        take_tokens.push(token);
                        pts += 1;
                    }
                    if pts < target {
                        for &area in &pool_cities {
                            if pts >= target { break; }
                            take_cities.push(area);
                            pts += 5;
                        }
                    }
                    state.beneficiary_selected_units = take_tokens;
                    state.beneficiary_selected_cities = take_cities;
                    info!("[CIVIL_WAR] AI beneficiary takes {} pts", pts);
                    finalize_first_faction(state, victim_areas, victim_cities);
                }
            }
            CivilWarPhase::VictimChoosesFaction => {
                if !state.has_second_faction() {
                    // Rule 30.413: no second faction means no Civil War --
                    // the calamity fizzles, nothing on the board changes.
                    info!("[CIVIL_WAR] First faction consumed the whole board -- no second faction, no Civil War (30.413)");
                    state.phase = CivilWarPhase::Complete;
                    continue;
                }

                if victim_is_human && cw_selection.acting_player.is_none() {
                    cw_selection.populate_faction_choice(
                        player_entity,
                        state.first_faction_points(),
                        state.second_faction_points(),
                    );
                    commands.entity(player_entity).insert(AwaitingHumanCalamitySelection);
                } else if victim_is_human && victim_awaiting {
                    // Still waiting for human UI
                } else if victim_is_human && !victim_awaiting && cw_selection.acting_player == Some(player_entity) {
                    let choice = cw_selection.take_faction_choice().unwrap_or(FactionChoice::First);
                    info!("[CIVIL_WAR] Human victim keeps {:?} faction (30.415)", choice);
                    state.kept_faction = Some(choice);
                    state.phase = CivilWarPhase::TransferFaction;
                } else {
                    let choice = state.default_ai_faction_choice();
                    info!("[CIVIL_WAR] AI victim keeps {:?} faction (30.415)", choice);
                    state.kept_faction = Some(choice);
                    state.phase = CivilWarPhase::TransferFaction;
                }
            }
            CivilWarPhase::TransferFaction => {
                let Some(beneficiary) = state.beneficiary else {
                    state.phase = CivilWarPhase::Complete;
                    continue;
                };

                // Rule 30.415: NO population token ever changes owner. ALL
                // tokens in the faction the victim did NOT keep go back to
                // the victim's own stock; they are then *replaced* in place
                // by the beneficiary's own tokens drawn from the
                // beneficiary's stock. The faction the victim keeps is left
                // completely untouched -- it was never enumerated as
                // "selected" in the first place from the board's point of view.
                //
                // The previous code reassigned `Token::player` to the beneficiary
                // and left the token in the victim's Population/PlayerAreas, which
                // created ghost tokens (owner != location), corrupting the token
                // pool and stalling later phases.
                let victim = player_entity;

                // Capture each selected token's current area from the victim's
                // PlayerAreas before we mutate anything.
                let mut token_area: bevy::platform::collections::HashMap<Entity, Entity> =
                    bevy::platform::collections::HashMap::default();
                for (area, tokens) in victim_areas.areas_and_population() {
                    for t in tokens {
                        token_area.insert(t, area);
                    }
                }

                // The transferring faction is fully enumerated now (30.415),
                // so every transferring token is both returned to stock and
                // replaced by a beneficiary token -- no partial-return bucket.
                let all_selected = state.transferring_units();
                let taken = all_selected.clone();
                let transferring_cities = state.transferring_cities();
                let returned_count = all_selected.len();
                let replaced_count = taken.len();

                commands.queue(move |world: &mut World| {
                    use bevy::platform::collections::{HashMap, HashSet};
                    let mut touched: HashSet<Entity> = HashSet::default();

                    // 1. Return every selected victim token to the victim's stock:
                    //    off the board (area Population + victim PlayerAreas), drop
                    //    visuals, back into stock. Owner stays the victim throughout.
                    for &token in &all_selected {
                        if let Some(&area) = token_area.get(&token) {
                            if let Some(mut pop) = world.get_mut::<Population>(area) {
                                pop.remove_token_from_area(victim, token);
                            }
                            touched.insert(area);
                        }
                        if let Some(mut pa) = world.get_mut::<PlayerAreas>(victim) {
                            pa.remove_token(token);
                        }
                        if let Some(mut stock) = world.get_mut::<TokenStock>(victim) {
                            stock.return_token_to_stock(token);
                        }
                        world
                            .entity_mut(token)
                            .remove::<(Sprite, Transform, Visibility)>();
                    }

                    // 2. Replace the taken tokens with the beneficiary's own
                    //    tokens from stock, in the same areas.
                    let ben_texture = world
                        .get::<Faction>(beneficiary)
                        .map(|f| f.faction)
                        .and_then(|f| {
                            world
                                .get_resource::<crate::civilization::concepts::map::map_plugin::AvailableFactions>()
                                .and_then(|af| af.faction_icons.get(&f).cloned())
                        });

                    let mut per_area: HashMap<Entity, usize> = HashMap::default();
                    for &token in &taken {
                        if let Some(&area) = token_area.get(&token) {
                            *per_area.entry(area).or_insert(0) += 1;
                        }
                    }
                    for (area, count) in per_area {
                        let area_pos = world
                            .get::<Transform>(area)
                            .map(|t| t.translation)
                            .unwrap_or_default();
                        for _ in 0..count {
                            let ben_token = world
                                .get_mut::<TokenStock>(beneficiary)
                                .and_then(|mut s| s.remove_token_from_stock());
                            let Some(ben_token) = ben_token else { break };
                            if let Some(mut pop) = world.get_mut::<Population>(area) {
                                pop.add_token_to_area(beneficiary, ben_token);
                            }
                            if let Some(mut pa) = world.get_mut::<PlayerAreas>(beneficiary) {
                                pa.add_token_to_area(area, ben_token);
                            }
                            let mut e = world.entity_mut(ben_token);
                            if let Some(tex) = ben_texture.clone() {
                                e.insert((
                                    Sprite { image: tex, ..default() },
                                    Transform::from_scale(Vec3::splat(0.25))
                                        .with_translation(area_pos),
                                ));
                            }
                        }
                        touched.insert(area);
                    }

                    for area in touched {
                        world.entity_mut(area).insert(FixTokenPositions);
                    }
                });

                // Cities: transfer every city in the transferring faction to
                // the beneficiary. Cities in the kept faction are untouched.
                for &area in &transferring_cities {
                    commands.entity(area).insert(TransferCityTo(beneficiary));
                }

                info!(
                    "[CIVIL_WAR] Transfer complete: {} victim tokens returned to stock, {} replaced by beneficiary",
                    returned_count, replaced_count
                );
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
        &mut player_query
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
                    let city_area = player_cities.areas_and_cities.keys().next().copied();
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
                        player_cities.areas_and_cities.keys().copied().collect();
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

/// Rule 30.911/30.913: finds the single, persistent, shared Pirate-nation
/// owner entity, or spawns one the first time any Piracy resolves this game.
/// Gives it exactly the components `transfer_city_to_new_owner` and the real
/// Conflict-phase city-combat machinery already require of any city owner --
/// `CityTokenStock`, `TokenStock`, `PlayerCities`, `PlayerAreas` -- but
/// deliberately no `Player`/`Treasury`/`PlayerTradeCards`, which is what
/// naturally exempts it from taxation, trading, and card draws. See
/// `PiracyState`'s doc comment for the full design.
///
/// KNOWN MINOR EDGE CASE: if two different players both resolve Piracy as
/// their own primary calamity in the exact same `advance_piracy` call (both
/// hitting `EnsurePirateNation` before either's spawn command has flushed),
/// each would independently see no existing `PirateNation` and spawn one --
/// resulting in two functionally-identical Pirate nations instead of one.
/// Harmless (neither behaves any differently), just inelegant; not worth a
/// synchronization mechanism for how rare simultaneous Piracy resolution is.
fn ensure_pirate_nation(commands: &mut Commands, existing: &Query<Entity, With<PirateNation>>) -> Entity {
    const CITY_STOCK: usize = 20;
    const TOKEN_STOCK: usize = 60;

    if let Some(e) = existing.iter().next() {
        return e;
    }
    let pirate_entity = commands.spawn((Name::new("Pirate Nation"), PirateNation)).id();
    let city_tokens: Vec<Entity> = (0..CITY_STOCK)
        .map(|_| commands.spawn((Name::new("Pirate City"), CityToken::new(pirate_entity))).id())
        .collect();
    let tokens: Vec<Entity> = (0..TOKEN_STOCK)
        .map(|_| commands.spawn((Name::new("Pirate"), Token::new(pirate_entity))).id())
        .collect();
    commands.entity(pirate_entity).insert((
        CityTokenStock::new(CITY_STOCK, city_tokens),
        TokenStock::new(TOKEN_STOCK, tokens),
        PlayerCities::default(),
        PlayerAreas::default(),
    ));
    info!("[PIRACY] Spawned the Pirate nation ({:?})", pirate_entity);
    pirate_entity
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
    pirate_nation_query: Query<Entity, With<PirateNation>>,
    mut calamity_resolved: MessageWriter<CalamityResolved>,
    mut calamity_selection: ResMut<CalamitySelectionState>,
) {
    for (player_entity, mut resolving, mut resolution, player_cities, is_human, awaiting_human) in &mut player_query {
        if resolution.phase == CalamityPhase::Resolved { continue; }
        let ResolvingCalamity::Piracy(ref mut state) = *resolving else { continue };

        match state.phase {
            PiracyPhase::EnsurePirateNation => {
                let pirate_entity = ensure_pirate_nation(&mut commands, &pirate_nation_query);
                state.pirate_nation = Some(pirate_entity);
                state.phase = PiracyPhase::SelectCoastalCities;
            }
            PiracyPhase::SelectCoastalCities => {
                // Rule 30.911: the trading player selects which 2 of the
                // primary victim's coastal cities are lost (replaced by
                // Pirate cities in ApplyEffects). Interactive selection for
                // the trading player is out of scope for this pass (same
                // simplification pattern as Barbarian Hordes' tie-breaking);
                // auto-selects the first 2 coastal cities in iteration order.
                let coastal_cities: Vec<Entity> = player_cities.areas_and_cities.keys()
                    .filter(|&&area| sea_passage_query.get(area).unwrap_or(false))
                    .copied()
                    .take(2)
                    .collect();
                let cities = if coastal_cities.is_empty() {
                    player_cities.areas_and_cities.keys().copied().take(2).collect()
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
                                .copied()
                                .collect();
                            if coastal.is_empty() {
                                cities.areas_and_cities.keys().take(1).copied().collect::<Vec<_>>()
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
                // Rule 30.911/30.912: both the primary victim's 2 coastal
                // cities and each secondary victim's 1 coastal city become
                // Pirate cities -- transferred to the shared PirateNation
                // entity (resolved in EnsurePirateNation, at least one phase
                // ago, so its components have already flushed), not to a
                // beneficiary player. transfer_city_to_new_owner (a regular
                // scheduled system) handles the actual BuiltCity/PlayerCities
                // bookkeeping identically to any other TransferCityTo target.
                if let Some(pirate_nation) = state.pirate_nation {
                    for &area in &state.cities_to_replace {
                        if area_query.get(area).is_ok() {
                            commands.entity(area).insert(TransferCityTo(pirate_nation));
                        }
                    }
                    for &area in &state.secondary_cities {
                        if area_query.get(area).is_ok() {
                            commands.entity(area).insert(TransferCityTo(pirate_nation));
                        }
                    }
                }
                info!(
                    "[PIRACY] Primary: {} cities, Secondary: {} cities replaced by Pirate cities",
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
/// are immune (rule 32.952).
///
/// AI holders are auto-processed immediately. Human holders pause: `MonotheismSelectionState`
/// is populated and `AwaitingMonotheismSelection` is inserted; the UI lets the human choose.
/// Once all holders are done the system transitions to `CheckCitySupportAfterResolveCalamities`.
pub fn apply_monotheism_conversions(
    mut commands: Commands,
    monotheism_holders: Query<(Entity, &PlayerAreas), With<NeedsMonotheismConversion>>,
    awaiting_query: Query<Entity, With<AwaitingMonotheismSelection>>,
    human_query: Query<(), With<IsHuman>>,
    all_players_civ: Query<(Entity, &PlayerCivilizationCards)>,
    land_passage_query: Query<&LandPassage>,
    population_query: Query<&Population>,
    mut mono_state: ResMut<MonotheismSelectionState>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // All done → transition.
    if monotheism_holders.is_empty() && awaiting_query.is_empty() {
        info!("[MONOTHEISM] All conversions done, transitioning to CheckCitySupport");
        next_state.set(GameActivity::CheckCitySupportAfterResolveCalamities);
        return;
    }

    // Theology holders are immune (rule 32.952).
    let theology_immune: bevy::platform::collections::HashSet<Entity> = all_players_civ
        .iter()
        .filter(|(_, c)| c.owns(&CivCardName::Theology))
        .map(|(e, _)| e)
        .collect();

    for (holder_entity, holder_areas) in monotheism_holders.iter() {
        let is_human = human_query.get(holder_entity).is_ok();
        let is_waiting = awaiting_query.get(holder_entity).is_ok();

        // Collect (token, area) candidates for this holder.
        let mut candidates: Vec<(Entity, Entity)> = Vec::new();
        'outer: for &held_area in &holder_areas.areas() {
            if let Ok(passages) = land_passage_query.get(held_area) {
                for &adj_area in &passages.to_areas {
                    if let Ok(pop) = population_query.get(adj_area) {
                        for (&enemy_player, tokens) in pop.player_tokens() {
                            if enemy_player == holder_entity || theology_immune.contains(&enemy_player) {
                                continue;
                            }
                            for &token in tokens {
                                candidates.push((token, adj_area));
                                if candidates.len() >= 2 {
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_human {
            if is_waiting {
                // Still waiting for the human to confirm.
                continue;
            }
            if mono_state.player == Some(holder_entity) {
                // Human just confirmed (AwaitingMonotheismSelection removed by button handler).
                let selected = mono_state.take_result();
                for token in selected {
                    commands.entity(token).insert(ReturnTokenToStock);
                    info!("[MONOTHEISM] Human eliminated token {:?}", token);
                }
                commands.entity(holder_entity).remove::<NeedsMonotheismConversion>();
            } else if mono_state.player.is_none() {
                // First time for this human holder: set up UI.
                mono_state.populate(holder_entity, candidates);
                commands.entity(holder_entity).insert(AwaitingMonotheismSelection);
                info!("[MONOTHEISM] Human player {:?} selecting targets", holder_entity);
            }
        } else {
            // AI: auto-select up to 2 candidates.
            for (token, _) in candidates.into_iter().take(2) {
                commands.entity(token).insert(ReturnTokenToStock);
                info!("[MONOTHEISM] {:?} eliminates token {:?}", holder_entity, token);
            }
            commands.entity(holder_entity).remove::<NeedsMonotheismConversion>();
        }
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
            .map_or_else(|_| "Unknown".to_string(), std::string::ToString::to_string);
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
    for (area_entity, mut population) in &mut areas_to_clear {
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
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut move_tokens: MessageWriter<crate::civilization::events::MoveTokensFromStockToAreaCommand>,
) {
    for (area_entity, built_city, population) in areas_with_reduce.iter() {
        info!("[CALAMITIES] Reducing city in area {:?}", area_entity);

        if let Ok((mut city_stock, mut player_cities, _, _)) =
            city_stock_query.get_mut(built_city.player)
        {
            player_cities.remove_city_from_area(area_entity);
            city_stock.return_token_to_stock(built_city.city);

            // Rule 26.11/26.6: calamity-driven reductions follow "the same
            // procedure" as support-driven ones (see eliminate_city in
            // check_city_support_systems.rs), including Agriculture's +1 --
            // the area is solely occupied by the reducing player's own
            // replacement tokens afterward, so the bonus always applies.
            // The pre-existing .min(6) cap already anticipated a max area
            // population of 5 (the highest value on the real map) plus this
            // +1, so it's left as-is rather than raised.
            let has_agriculture = civ_cards_query
                .get(built_city.player)
                .is_ok_and(|c| c.owns(&CivCardName::Agriculture));
            move_tokens.write(
                crate::civilization::events::MoveTokensFromStockToAreaCommand {
                    player_entity: built_city.player,
                    area_entity,
                    number_of_tokens: (population.max_population + usize::from(has_agriculture)).min(6),
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
                    .insert(BuiltCity::new(new_city, new_owner));
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
