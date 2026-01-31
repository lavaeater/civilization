use crate::civilization::components::population::Population;
use crate::civilization::components::BuiltCity;
use crate::civilization::components::ReturnTokenToStock;
use crate::GameActivity;
use bevy::prelude::{Commands, Has, Name, NextState, Query, ResMut};

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut areas: Query<(&mut Population, Has<BuiltCity>)>,
    _name_query: Query<&Name>,
    mut commands: Commands,
) {
    for (mut area, has_city) in areas.iter_mut() {
        if area.has_surplus(has_city) {
            if has_city {
                //debug!("Area has a city, so we remove all tokens");
                for token in area.remove_all_tokens() {
                    commands.entity(token).insert(ReturnTokenToStock);
                }
            } else if area.number_of_players() > 1 {
                for (_player, _tokens) in area.player_tokens().iter() {
                    //debug!("Player {:#?} has {:#?} tokens", name_query.get(*player), tokens.len());
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
