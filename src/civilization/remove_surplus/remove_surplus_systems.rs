use bevy::prelude::{debug, EventWriter, NextState, Query, ResMut};
use crate::civilization::general::general_components::Population;
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;

pub fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut areas: Query<&mut Population>,
) {
    for mut area in areas.iter_mut() {
        if area.has_surplus() {
            if area.number_of_players() > 1 {
                // debug!("Area {:?} has surplus population", area);
            } else {
                for token in area.remove_surplus() {
                    return_token.send(ReturnTokenToStock {
                        token_entity: token,
                    });
                }
                // debug!("Removed surplus population from area {:?}", area);
            }
        }
    }
    next_state.set(GameActivity::CheckCitySupport);
}