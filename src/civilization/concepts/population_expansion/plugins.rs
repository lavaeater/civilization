use crate::civilization::concepts::population_expansion::components::ExpandAutomatically;
use crate::civilization::concepts::population_expansion::events::{CheckGate, CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand};
use crate::civilization::concepts::population_expansion::systems::{auto_expand_population, check_area_population_expansion_eligibility, enter_population_expansion, expand_population_manually, population_expansion_gate};
use crate::civilization::concepts::population_expansion::triggers::on_remove_needs_expansion;
use crate::GameActivity;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter};

pub struct PopulationExpansionPlugin;

impl Plugin for PopulationExpansionPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ExpandAutomatically>()
            .add_event::<CheckPlayerExpansionEligibility>()
            .add_event::<ExpandPopulationManuallyCommand>()
            .add_event::<CheckGate>()
            .add_systems(OnEnter(GameActivity::PopulationExpansion),
                         enter_population_expansion)
            .add_systems(
                Update, (
                    auto_expand_population
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    expand_population_manually
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    population_expansion_gate
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    check_area_population_expansion_eligibility
                        .run_if(in_state(GameActivity::PopulationExpansion))
                ),
            )
            .add_observer(on_remove_needs_expansion)
        ;
    }
}