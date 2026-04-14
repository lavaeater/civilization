use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update};

use crate::civilization::concepts::resolve_calamities::calamities::ResolvingCalamity;
use crate::civilization::concepts::resolve_calamities::context::ActiveCalamityResolution;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_components::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_components::{
    AwaitingHumanCalamitySelection, AwaitingMonotheismSelection, CalamitySelectionState,
    CivilWarSelectionState, MonotheismSelectionState,
};
use crate::civilization::concepts::resolve_calamities::resolve_calamities_ui_systems::*;
use crate::civilization::resolve_calamities::resolve_calamities_events::{
    CalamityResolved, Earthquake, ResolveNextCalamity, ResolveVolcanoEarthquake, VolcanoEruption,
};

pub struct ResolveCalamitiesPlugin;

impl Plugin for ResolveCalamitiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<ResolveNextCalamity>()
            .add_message::<ResolveVolcanoEarthquake>()
            .add_message::<VolcanoEruption>()
            .add_message::<Earthquake>()
            .add_message::<CalamityResolved>()
            .register_type::<NeedsCalamityResolution>()
            .register_type::<NeedsMonotheismConversion>()
            .register_type::<AwaitingHumanCalamitySelection>()
            .register_type::<AwaitingMonotheismSelection>()
            .register_type::<CalamityVictim>()
            .register_type::<PendingCalamities>()
            .register_type::<ActiveCalamityResolution>()
            .register_type::<ResolvingCalamity>()
            .register_type::<ClearAllTokens>()
            .register_type::<DestroyCity>()
            .register_type::<ReduceCity>()
            .register_type::<TransferCityTo>()
            .register_type::<ReturnCityToStock>()
            .init_resource::<CalamitySelectionState>()
            .init_resource::<CivilWarSelectionState>()
            .init_resource::<MonotheismSelectionState>()
            .add_systems(
                OnEnter(GameActivity::ResolveCalamities),
                start_calamity_resolution,
            )
            .add_systems(
                OnExit(GameActivity::ResolveCalamities),
                cleanup_calamity_selection_ui_on_exit,
            )
            .add_systems(
                Update,
                (
                    // Dispatch: picks the next pending calamity and sets up its state machine
                    process_pending_calamities,
                    // VolcanoEarthquake (two-step via message)
                    resolve_volcano_earthquake,
                    apply_volcano_earthquake_effects,
                    // Flood
                    advance_flood,
                    // Unit-point-loss calamities
                    advance_famine,
                    advance_barbarian_hordes,
                    advance_epidemic,
                    advance_iconoclasm_heresy,
                    // City-reduction calamities
                    advance_superstition,
                    advance_slave_revolt,
                    advance_civil_disorder,
                ).run_if(in_state(GameActivity::ResolveCalamities)),
            )
            .add_systems(
                Update,
                (
                    // Multi-player / complex calamities
                    advance_civil_war,
                    advance_treachery,
                    advance_piracy,
                    // Post-resolution
                    apply_monotheism_conversions,
                    handle_calamity_resolved,
                    clear_all_tokens_from_area,
                    destroy_city_in_area,
                    reduce_city_in_area,
                    transfer_city_to_new_owner,
                    // Human calamity selection UI
                    spawn_calamity_selection_ui,
                    update_calamity_selection_ui,
                    handle_calamity_selection_buttons,
                    cleanup_calamity_selection_ui,
                    // Civil War human selection UI
                    spawn_civil_war_selection_ui,
                    update_civil_war_selection_ui,
                    handle_civil_war_selection_buttons,
                    cleanup_civil_war_selection_ui,
                ).run_if(in_state(GameActivity::ResolveCalamities)),
            )
            .add_systems(
                Update,
                (
                    // Monotheism human target-selection UI
                    spawn_monotheism_selection_ui,
                    update_monotheism_selection_ui,
                    cleanup_monotheism_selection_ui,
                ).run_if(in_state(GameActivity::ResolveCalamities)),
            );
    }
}
