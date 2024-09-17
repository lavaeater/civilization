use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter};
use crate::civilization::population_expansion::population_expansion_components::ExpandAutomatically;
use crate::civilization::population_expansion::population_expansion_events::{CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand};
use crate::civilization::population_expansion::population_expansion_systems::{check_area_population_expansion_eligibility, check_population_expansion_eligibility, expand_population, expand_population_manually, population_expansion_gate};
use crate::GameActivity;

pub struct PopulationExpansionPlugin;

impl Plugin for PopulationExpansionPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ExpandAutomatically>()
            .add_event::<CheckPlayerExpansionEligibility>()
            .add_event::<ExpandPopulationManuallyCommand>()
            .add_systems(OnEnter(GameActivity::PopulationExpansion),
                         check_population_expansion_eligibility)
            .add_systems(
                Update, (
                    expand_population
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    expand_population_manually
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    population_expansion_gate
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    check_area_population_expansion_eligibility
                        .run_if(in_state(GameActivity::PopulationExpansion))
                ),
            )
        ;
    }
}