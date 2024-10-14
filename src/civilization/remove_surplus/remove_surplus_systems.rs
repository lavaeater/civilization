use crate::civilization::general::general_components::{BuiltCity, Population};
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;
use bevy::prelude::{debug, EventWriter, Has, NextState, Query, ResMut};

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut areas: Query<(&mut Population, Has<BuiltCity>)>,
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
                debug!("More than two players and city: {}, total_pop: {}, max_pop: {}", area.number_of_players(), area.total_population(), area.max_population);
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