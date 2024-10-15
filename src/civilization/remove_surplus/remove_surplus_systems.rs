use crate::civilization::general::general_components::population::Population;
use crate::civilization::general::general_components::*;
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;
use bevy::prelude::{debug, EventWriter, Has, Name, NextState, Query, ResMut};

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut areas: Query<(&mut Population, Has<BuiltCity>)>,
    name_query: Query<&Name>
) {
    for (mut area, has_city) in areas.iter_mut() {
        if area.has_surplus(has_city) {
            if has_city {
                debug!("Area has a city, so we remove all tokens");
                for token in area.remove_all_tokens() {
                    return_token.send(ReturnTokenToStock {
                        token_entity: token,
                    });
                }
            } else if area.number_of_players() > 1 {
                for (player, tokens) in area.player_tokens().iter() {
                    debug!("Player {:?} has {:?} tokens", name_query.get(*player), tokens.len());
                }
            } else {
                for token in area.remove_surplus() {
                    return_token.send(ReturnTokenToStock {
                        token_entity: token,
                    });
                }
            }
        }
    }
    next_state.set(GameActivity::CheckCitySupport);
}