use bevy::prelude::{EventWriter, NextState, Query, ResMut};
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
            
            println!("Removed surplus population from area {:?}", area);
        }
    }
    next_state.set(GameActivity::CheckCitySupport);
}