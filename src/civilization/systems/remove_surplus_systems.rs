use crate::GameActivity;
use bevy::prelude::{debug, Commands, Has, Name, NextState, Query, ResMut};
use crate::civilization::components::prelude::{BuiltCity, Population};
use crate::civilization::triggers::prelude::ReturnTokenToStock;

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut areas: Query<(&mut Population, Has<BuiltCity>)>,
    name_query: Query<&Name>,
    mut commands: Commands
) {
    for (mut area, has_city) in areas.iter_mut() {
        if area.has_surplus(has_city) {
            if has_city {
                debug!("Area has a city, so we remove all tokens");
                for token in area.remove_all_tokens() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            } else if area.number_of_players() > 1 {
                for (player, tokens) in area.player_tokens().iter() {
                    debug!("Player {:?} has {:?} tokens", name_query.get(*player), tokens.len());
                }
            } else {
                for token in area.remove_surplus() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            }
        }
    }
    next_state.set(GameActivity::CheckCitySupport);
}