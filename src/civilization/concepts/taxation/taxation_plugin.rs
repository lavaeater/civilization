use crate::civilization::concepts::taxation::taxation_components::{
    CityInRevolt, NeedsToPayTaxes,
};
use crate::civilization::concepts::taxation::taxation_systems::{
    collect_taxes, enter_collect_taxes, resolve_revolts, taxation_gate,
};
use crate::GameActivity;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter};

pub struct TaxationPlugin;

impl Plugin for TaxationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NeedsToPayTaxes>()
            .register_type::<CityInRevolt>()
            .add_systems(OnEnter(GameActivity::CollectTaxes), enter_collect_taxes)
            .add_systems(
                Update,
                (
                    collect_taxes.run_if(in_state(GameActivity::CollectTaxes)),
                    resolve_revolts.run_if(in_state(GameActivity::CollectTaxes)),
                    taxation_gate.run_if(in_state(GameActivity::CollectTaxes)),
                )
                    .chain(),
            );
    }
}
