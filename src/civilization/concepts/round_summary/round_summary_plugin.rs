use crate::GameActivity;
use crate::civilization::concepts::round_summary::round_summary_components::RoundSummary;
use bevy::prelude::{App, OnEnter, Plugin, ResMut};

/// Registers `RoundSummary` and clears it at the start of every round.
pub struct RoundSummaryPlugin;

impl Plugin for RoundSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoundSummary>()
            .add_systems(OnEnter(GameActivity::CollectTaxes), clear_round_summary);
    }
}

fn clear_round_summary(mut summary: ResMut<RoundSummary>) {
    summary.clear();
}
