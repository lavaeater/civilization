use crate::civilization::components::{BuiltCity, Population, ReturnTokenToStock};
use crate::GameActivity;
use bevy::prelude::{Commands, Has, NextState, Query, ResMut};

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut areas: Query<(&mut Population, Has<BuiltCity>)>,
    mut commands: Commands,
) {
    for (mut area, has_city) in areas.iter_mut() {
        if area.has_surplus(has_city) {
            if has_city {
                for token in area.remove_all_tokens() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            } else if area.number_of_players() > 1 {
                remove_surplus_multi_player(&mut area, &mut commands);
            } else {
                for token in area.remove_surplus() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            }
        }
    }
    next_state.set(GameActivity::CheckCitySupport);
}

/// Remove surplus tokens from an area with multiple players.
/// Removes one token at a time from the player with the fewest tokens
/// until total population is at or below max_population.
fn remove_surplus_multi_player(area: &mut Population, commands: &mut Commands) {
    while area.total_population() > area.max_population && area.number_of_players() > 0 {
        // Find the player with the fewest tokens
        let smallest_player = area
            .player_tokens()
            .iter()
            .min_by_key(|(_, tokens)| tokens.len())
            .map(|(player, _)| *player);

        if let Some(player) = smallest_player {
            if let Some(removed) = area.remove_tokens_from_area(&player, 1) {
                for token in removed {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
}
