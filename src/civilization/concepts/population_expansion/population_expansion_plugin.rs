use crate::civilization::concepts::population_expansion::population_expansion_components::{
    ExpandAutomatically, PopExpAreaHighlight, PopExpHighlightMarker,
};
use crate::civilization::concepts::population_expansion::population_expansion_events::{
    CheckGate, CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand,
};
use crate::civilization::concepts::population_expansion::population_expansion_systems::{
    auto_expand_population, check_area_population_expansion_eligibility,
    cleanup_pop_exp_highlights, enter_population_expansion, expand_population_manually,
    handle_pop_exp_area_click, highlight_pop_exp_areas_for_human, log_round_state,
    population_expansion_gate,
};
use crate::civilization::concepts::population_expansion::population_expansion_triggers::on_remove_needs_expansion;
use crate::civilization::general_systems::move_tokens_from_stock_to_area;
use crate::GameActivity;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter};

pub struct PopulationExpansionPlugin;

impl Plugin for PopulationExpansionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ExpandAutomatically>()
            .register_type::<PopExpAreaHighlight>()
            .register_type::<PopExpHighlightMarker>()
            .add_message::<CheckPlayerExpansionEligibility>()
            .add_message::<ExpandPopulationManuallyCommand>()
            .add_message::<CheckGate>()
            .add_systems(
                OnEnter(GameActivity::PopulationExpansion),
                (enter_population_expansion, log_round_state).chain(),
            )
            .add_systems(
                Update,
                (
                    // Deterministic order around move_tokens_from_stock_to_area
                    // (added once by the civilization plugin): the eligibility
                    // check must see the token stock *after* pending token
                    // moves executed, otherwise it re-inserts ExpandManually
                    // from a stale stock and offers 0-token moves while the
                    // gate never fires. The AI used to self-heal by submitting
                    // those 0-token moves; remote human players cannot.
                    auto_expand_population
                        .before(move_tokens_from_stock_to_area)
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    expand_population_manually
                        .before(move_tokens_from_stock_to_area)
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    check_area_population_expansion_eligibility
                        .after(move_tokens_from_stock_to_area)
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    population_expansion_gate
                        .after(check_area_population_expansion_eligibility)
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    highlight_pop_exp_areas_for_human
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    handle_pop_exp_area_click
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    cleanup_pop_exp_highlights
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                ),
            )
            .add_observer(on_remove_needs_expansion);
    }
}
