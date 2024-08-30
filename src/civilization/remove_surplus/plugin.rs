use bevy::prelude::{App, EventWriter, NextState, OnEnter, Plugin, Query, ResMut};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::Population;
use crate::civilization::general::events::ReturnTokenToStock;

pub struct RemoveSurplusPlugin;

impl Plugin for RemoveSurplusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameActivity::RemoveSurplusPopulation), remove_surplus_population)
        ;
    }
}

fn remove_surplus_population(
    mut next_state: ResMut<NextState<GameActivity>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut areas: Query<&mut Population>,
) {
    for mut area in areas.iter_mut() {
        let surplus: i32 = area.total_population.try_into().unwrap_or(i32::MAX) - area.max_population.try_into().unwrap_or(i32::MAX);
        if surplus > 0 {
            assert_eq!(area.player_tokens.keys().len(), 1); // this should never, ever, happen
            area.player_tokens.values_mut().next().unwrap().drain(0..surplus.try_into().unwrap_or(0)).for_each(|token| {
                return_token.send(ReturnTokenToStock {
                    token_entity: token,
                });
            });
            area.total_population = area.max_population;
            println!("Removed {} surplus population from area {:?}", surplus, area);
        }
    }
    next_state.set(GameActivity::PopulationExpansion);
}