use crate::civilization::components::{BuiltCity, GameArea, Population, ReturnTokenToStock};
use crate::GameActivity;
use bevy::prelude::{info, warn, Commands, Entity, Has, Name, NextState, Query, ResMut, With};

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut areas: Query<(Entity, &Name, &mut Population, Has<BuiltCity>), With<GameArea>>,
    mut commands: Commands,
) {
    let mut total_areas_processed = 0;
    let mut areas_with_surplus = 0;
    let mut total_tokens_removed = 0;
    
    for (_area_entity, name, mut area, has_city) in areas.iter_mut() {
        total_areas_processed += 1;
        let total_pop = area.total_population();
        let max_pop = area.max_population;
        let surplus = area.surplus_count();
        let num_players = area.number_of_players();
        
        if area.has_surplus(has_city) {
            areas_with_surplus += 1;
            info!(
                "[SURPLUS] '{}' has surplus: total={}, max={}, surplus={}, players={}, has_city={}",
                name, total_pop, max_pop, surplus, num_players, has_city
            );
            
            let tokens_before = area.total_population();
            
            if has_city {
                info!("[SURPLUS] '{}': Removing all tokens from area with city", name);
                let tokens = area.remove_all_tokens();
                for token in tokens.iter() {
                    commands.entity(*token).insert(ReturnTokenToStock);
                }
                total_tokens_removed += tokens.len();
            } else if num_players > 1 {
                info!("[SURPLUS] '{}': Removing surplus tokens from area with {} players", name, num_players);
                let removed = remove_surplus_multi_player(&mut area, &mut commands);
                total_tokens_removed += removed;
            } else if num_players == 1 {
                info!("[SURPLUS] '{}': Removing surplus tokens from single-player area", name);
                let tokens = area.remove_surplus();
                for token in tokens.iter() {
                    commands.entity(*token).insert(ReturnTokenToStock);
                }
                total_tokens_removed += tokens.len();
            } else {
                warn!("[SURPLUS] '{}': No players in area but has_surplus returned true - clearing any stale data", name);
            }
            
            let tokens_after = area.total_population();
            let remaining_surplus = area.surplus_count();
            
            info!(
                "[SURPLUS] '{}': After removal: {} -> {} tokens, surplus={}",
                name, tokens_before, tokens_after, remaining_surplus
            );
            
            // Safety check: if we still have surplus after removal, log a warning
            if remaining_surplus > 0 && !has_city {
                warn!(
                    "[SURPLUS] '{}': Still has {} surplus after removal! This may indicate a bug.",
                    name, remaining_surplus
                );
            }
        }
    }
    
    info!(
        "[SURPLUS] Phase complete: processed {} areas, {} had surplus, removed {} tokens total",
        total_areas_processed, areas_with_surplus, total_tokens_removed
    );
    
    next_state.set(GameActivity::CheckCitySupportAfterRemoveSurplusPopulation);
}

/// Remove surplus tokens from an area with multiple players.
/// Removes one token at a time from the player with the fewest tokens
/// until total population is at or below max_population.
/// Returns the number of tokens removed.
fn remove_surplus_multi_player(area: &mut Population, commands: &mut Commands) -> usize {
    let mut removed_count = 0;
    
    while area.total_population() > area.max_population && area.number_of_players() > 0 {
        // Find the player with the fewest tokens
        let smallest_player = area
            .player_tokens()
            .iter()
            .min_by_key(|(_, tokens)| tokens.len())
            .map(|(player, _)| *player);

        if let Some(player) = smallest_player {
            if let Some(removed) = area.remove_tokens_from_area(&player, 1) {
                for token in removed.iter() {
                    commands.entity(*token).insert(ReturnTokenToStock);
                }
                removed_count += removed.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    removed_count
}
