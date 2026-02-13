use crate::GameActivity;
use bevy::prelude::{in_state, App, IntoScheduleConfigs, OnEnter, Plugin, Update};

use crate::civilization::concepts::resolve_calamities::resolve_calamities_components::*;
use crate::civilization::concepts::resolve_calamities::resolve_calamities_systems::*;

pub struct ResolveCalamitiesPlugin;

impl Plugin for ResolveCalamitiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<NeedsCalamityResolution>()
            .register_type::<CalamityVictim>()
            .register_type::<PendingCalamities>()
            .register_type::<ResolvingVolcanoEarthquake>()
            .register_type::<ClearAllTokens>()
            .register_type::<DestroyCity>()
            .register_type::<ReduceCity>()
            .register_type::<ReturnCityToStock>()
            .add_systems(
                OnEnter(GameActivity::ResolveCalamities),
                start_calamity_resolution,
            )
            .add_systems(
                Update,
                (
                    process_pending_calamities,
                    resolve_volcano_earthquake,
                    handle_calamity_resolved,
                    clear_all_tokens_from_area,
                    destroy_city_in_area,
                    reduce_city_in_area,
                ).run_if(in_state(GameActivity::ResolveCalamities)),
            );
    }
}
