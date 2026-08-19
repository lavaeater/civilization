use crate::civilization::components::*;
use crate::civilization::concepts::acquire_trade_cards::PlayerTradeCards;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::conflict::conflict_components::*;
use crate::civilization::concepts::conflict::conflict_functions::*;
use crate::civilization::concepts::map::CameraFocusQueue;
use crate::civilization::functions::{replace_city_with_tokens_for_conflict, return_all_tokens_from_area_to_player};
use crate::civilization::{CivCardName, ConflictCounterResource};
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::platform::collections::HashSet;
use bevy::prelude::{Add, Commands, Entity, Name, NextState, On, Query, ResMut, Transform, With};
use bevy::log::info;
use std::cmp::Ordering;

pub fn on_add_unresolved_conflict(
    trigger: On<Add, UnresolvedConflict>,
    mut areas: Query<(Entity, &Name, &mut Population, &Transform)>,
    mut commands: Commands,
    mut conflict_counter_resource: ResMut<ConflictCounterResource>,
    mut next_state: ResMut<NextState<GameActivity>>,
    human_players: Query<Entity, With<IsHuman>>,
    mut camera_focus: ResMut<CameraFocusQueue>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
) {
    if let Ok((area_entity, name, mut population, transform)) = areas.get_mut(trigger.event().entity) {
        let player_count = population.number_of_players();
        let total_pop = population.total_population();
        let max_pop = population.max_population;

        // Check if human player is involved in this conflict
        let human_involved = human_players.iter().any(|human| population.has_player(&human));

        if human_involved {
            camera_focus.add_focus(
                transform.translation,
                1.5,
                format!("Conflict in {name}"),
            );
        }

        info!(
            "[CONFLICT] Regular conflict in '{}': {} players, {} tokens, max_pop={}, conflicts_pending={}, human_involved={}",
            name, player_count, total_pop, max_pop, conflict_counter_resource.0, human_involved
        );

        let temp_map = population.player_tokens().clone();
        let mut players = temp_map.keys().copied().collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        for (i, player) in players.iter().enumerate() {
            let token_count = temp_map.get(player).map_or(0, HashSet::len);
            info!("  Player {} ({:?}): {} tokens", i + 1, player, token_count);
        }

        // Build the set of Metalworking holders in this conflict (rule 24.24).
        let metalworking_set: HashSet<Entity> = players
            .iter()
            .filter(|&&p| {
                civ_cards_query
                    .get(p)
                    .is_ok_and(|c| c.owns(&CivCardName::Metalworking))
            })
            .copied()
            .collect();

        let resolution_type = if population.max_population == 1 {
            handle_max_pop_is_one_conflicts(&mut players, &mut population, &mut commands);
            "max_pop=1"
        } else if has_metalworking_mix(&players, &metalworking_set) {
            // Rule 24.24: non-Metalworking players remove tokens before MW holders.
            let mut non_mw: Vec<Entity> = players
                .iter()
                .filter(|p| !metalworking_set.contains(*p))
                .copied()
                .collect();
            let mut mw: Vec<Entity> = players
                .iter()
                .filter(|p| metalworking_set.contains(*p))
                .copied()
                .collect();
            info!("[CONFLICT] Metalworking rule applies: {} non-MW, {} MW players", non_mw.len(), mw.len());
            handle_with_metalworking(&mut non_mw, &mut mw, &mut population, &mut commands);
            "metalworking"
        } else if population.all_lengths_equal() {
            handle_all_lengths_equal(&players, &mut population, &mut commands);
            "all_lengths_equal"
        } else {
            handle_unequal_lengths(&mut players, &mut population, &mut commands);
            "unequal_lengths"
        };

        let remaining_pop = population.total_population();
        info!(
            "[CONFLICT] Resolved '{}' via {}: {} tokens remaining (was {})",
            name, resolution_type, remaining_pop, total_pop
        );

        commands.entity(area_entity).remove::<UnresolvedConflict>();
        conflict_counter_resource.0 = conflict_counter_resource.0.saturating_sub(1);
        info!("[CONFLICT] Conflicts remaining: {}", conflict_counter_resource.0);
        if conflict_counter_resource.0 == 0 {
            info!("[CONFLICT] All conflicts resolved, transitioning to CityConstruction");
            next_state.set(GameActivity::CityConstruction);
        }
    } else {
        info!("[CONFLICT] Failed to get area for entity {:?}", trigger.event().entity);
    }
}

pub fn on_add_unresolved_city_conflict(
    trigger: On<Add, UnresolvedCityConflict>,
    mut areas: Query<(Entity, &Name, &mut Population, &BuiltCity, &Transform)>,
    mut player_with_city: Query<(
        &mut CityTokenStock,
        &mut TokenStock,
        &mut PlayerCities,
        &mut PlayerAreas,
    )>,
    mut commands: Commands,
    mut conflict_counter_resource: ResMut<ConflictCounterResource>,
    mut next_state: ResMut<NextState<GameActivity>>,
    human_players: Query<Entity, With<IsHuman>>,
    mut camera_focus: ResMut<CameraFocusQueue>,
    civ_cards_query: Query<&PlayerCivilizationCards>,
    mut trade_cards_query: Query<&mut PlayerTradeCards>,
    mut treasury_query: Query<&mut Treasury>,
) {
    if let Ok((area_entity, name, mut population, built_city, transform)) = areas.get_mut(trigger.event().entity) {
        let total_pop = population.total_population();
        let max_pop = population.max_population;
        let city_owner = built_city.player;
        let owner_tokens = population.population_for_player(city_owner);
        
        // Check if human player is involved (either as city owner or invader)
        let human_is_owner = human_players.iter().any(|h| h == city_owner);
        let human_is_invader = human_players.iter().any(|human| population.has_player(&human) && human != city_owner);
        let human_involved = human_is_owner || human_is_invader;
        
        if human_involved {
            camera_focus.add_focus(
                transform.translation,
                1.5,
                format!("City conflict in {name}"),
            );
        }
        
        let mut other_players = population.players();
        other_players.remove(&built_city.player);
        
        info!(
            "[CITY CONFLICT] City conflict in '{}': owner={:?} ({} tokens), {} invaders, total={}, max_pop={}, conflicts_pending={}",
            name, city_owner, owner_tokens, other_players.len(), total_pop, max_pop, conflict_counter_resource.0
        );
        
        for player in &other_players {
            let token_count = population.population_for_player(*player);
            info!("  Invader {:?}: {} tokens", player, token_count);
        }

        // Rule 24.35: Engineering shifts the attack threshold (base 7, rule
        // 24.31) and the number of tokens the city is replaced by (base 6,
        // rule 24.32) depending on which side(s) hold it.
        let defender_has_engineering = civ_cards_query
            .get(built_city.player)
            .is_ok_and(|c| c.owns(&CivCardName::Engineering));
        let has_large_invader = other_players.iter().any(|p| {
            let attacker_has_engineering = civ_cards_query
                .get(*p)
                .is_ok_and(|c| c.owns(&CivCardName::Engineering));
            let (tokens_required, _) =
                attack_thresholds(attacker_has_engineering, defender_has_engineering);
            population.population_for_player(*p) >= tokens_required
        });

        if has_large_invader {
            info!("[CITY CONFLICT] Large invader detected");
            match other_players.len().cmp(&1) {
                Ordering::Less => {
                    info!("[CITY CONFLICT] No other players - edge case");
                }
                Ordering::Equal => {
                    let attacker = *other_players.iter().next().expect("len == 1");
                    let attacker_has_engineering = civ_cards_query
                        .get(attacker)
                        .is_ok_and(|c| c.owns(&CivCardName::Engineering));
                    let (_, replacement_count) =
                        attack_thresholds(attacker_has_engineering, defender_has_engineering);
                    info!("[CITY CONFLICT] Single large invader - replacing city with {} tokens", replacement_count);
                    if let Ok((
                        mut city_stock,
                        mut token_stock,
                        mut player_cities,
                        mut player_areas,
                    )) = player_with_city.get_mut(built_city.player)
                    {
                        commands.entity(area_entity).remove::<BuiltCity>();
                        replace_city_with_tokens_for_conflict(
                            area_entity,
                            &mut population,
                            built_city,
                            &mut city_stock,
                            &mut token_stock,
                            &mut player_cities,
                            &mut player_areas,
                            replacement_count,
                        );
                        info!("[CITY CONFLICT] City replaced, inserting UnresolvedConflict for regular resolution");
                        commands.entity(area_entity).insert(UnresolvedConflict);

                        // Rules 24.51-24.52: a city eliminated by direct attack
                        // (as opposed to Monotheism/Pirates/calamity/tax revolt,
                        // 24.53) costs its owner a random trade card and up to
                        // 3 pillaged tokens per city. This is unconditionally
                        // beneficial to the attacker, so it's auto-applied for
                        // both AI and human players; a "let the human decline"
                        // UI hook would be a pure nice-to-have on top of this.
                        //
                        // Rule 30.526: if Barbarian Hordes eliminates a city,
                        // NO trade card is drawn from the victim at all -- not
                        // just "the attacker doesn't receive it". Barbarians
                        // (see BarbarianHordesState's doc comment) have no
                        // PlayerTradeCards, so gating the whole draw on the
                        // attacker having one (checked first, before touching
                        // the victim's hand) implements this correctly rather
                        // than removing a card from the victim and discarding
                        // it, which the old attacker-only check would have done.
                        if trade_cards_query.get(attacker).is_ok()
                            && let Ok(mut victim_cards) = trade_cards_query.get_mut(built_city.player)
                            && let Some(drawn) = victim_cards.remove_random_card()
                        {
                            if let Ok(mut attacker_cards) = trade_cards_query.get_mut(attacker) {
                                attacker_cards.add_trade_card(drawn);
                                info!(
                                    "[CITY CONFLICT] Rule 24.51: {:?} drew {:?} from {:?}'s hand",
                                    attacker, drawn, built_city.player
                                );
                            } else {
                                info!("[CITY CONFLICT] Rule 24.51: attacker {:?} has no PlayerTradeCards, card lost", attacker);
                            }
                        }
                        if let Ok((_, mut attacker_stock, _, _)) = player_with_city.get_mut(attacker)
                            && let Ok(mut attacker_treasury) = treasury_query.get_mut(attacker)
                            && let Some(pillaged) = attacker_stock.remove_at_most_n_tokens_from_stock(3)
                        {
                            let pillaged_count = pillaged.len();
                            for token in pillaged {
                                attacker_treasury.add_token_to_treasury(token);
                            }
                            info!(
                                "[CITY CONFLICT] Rule 24.52: {:?} pillaged {} token(s) from stock to treasury",
                                attacker, pillaged_count
                            );
                        }
                    } else {
                        info!("[CITY CONFLICT] Failed to get city owner components");
                    }
                }
                Ordering::Greater => {
                    info!("[CITY CONFLICT] Multiple invaders - resolving battles between others first");
                    commands.entity(trigger.event().entity).insert(UnresolvedConflict);
                }
            }
        } else {
            info!("[CITY CONFLICT] Small invaders (<=6 tokens each) - eliminating them");
            for player in other_players {
                if let Ok((_, mut token_stock, _, mut player_areas)) =
                    player_with_city.get_mut(player)
                {
                    let removed = population.population_for_player(player);
                    return_all_tokens_from_area_to_player(
                        &player,
                        &area_entity,
                        &mut population,
                        &mut token_stock,
                        &mut player_areas,
                    );
                    info!("[CITY CONFLICT] Returned {} tokens to player {:?}", removed, player);
                } else {
                    info!("[CITY CONFLICT] Failed to get components for player {:?}", player);
                }
            }
        }
        
        let remaining_pop = population.total_population();
        info!("[CITY CONFLICT] After resolution: {} tokens remaining (was {})", remaining_pop, total_pop);
        
        commands
            .entity(area_entity)
            .remove::<UnresolvedCityConflict>();
        
        // Only decrement the counter if we did NOT chain into UnresolvedConflict.
        // When we insert UnresolvedConflict (large invader paths), that observer
        // will handle the decrement — so we must not double-decrement here.
        if has_large_invader {
            info!("[CITY CONFLICT] Chained into UnresolvedConflict — counter will be decremented by that observer");
        } else {
            conflict_counter_resource.0 = conflict_counter_resource.0.saturating_sub(1);
            info!("[CITY CONFLICT] Conflicts remaining: {}", conflict_counter_resource.0);
            if conflict_counter_resource.0 == 0 {
                info!("[CITY CONFLICT] All conflicts resolved, transitioning to CityConstruction");
                next_state.set(GameActivity::CityConstruction);
            }
        }
    } else {
        info!("[CITY CONFLICT] Failed to get area for entity {:?}", trigger.event().entity);
    }
}
