use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::population_expansion::components::ExpandAutomatically;
use crate::civilization::population_expansion::events::StartManualPopulationExpansionEvent;
use crate::civilization::population_expansion::systems::{check_population_expansion_eligibility, expand_population};

pub struct PopulationExpansionPlugin;

impl Plugin for PopulationExpansionPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ExpandAutomatically>()
            .add_event::<StartManualPopulationExpansionEvent>()
            .add_systems(OnEnter(GameActivity::PopulationExpansion),
                         check_population_expansion_eligibility)
            .add_systems(
                Update, (
                    expand_population
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                ),
            )
        ;
    }
}


/***
    * This system checks if the player has enough tokens to expand the population in all areas.
 */

/*
Later, bro
 */
// pub fn handle_manual_population_expansion(
//     mut start_reader: EventReader<StartManualPopulationExpansionEvent>,
//     mut expand_writer: EventWriter<BeginPopulationExpansionEvent>,
// ) {
//     for _start in start_reader.read() {
//         expand_writer.send(BeginPopulationExpansionEvent {});
//     }
// }

