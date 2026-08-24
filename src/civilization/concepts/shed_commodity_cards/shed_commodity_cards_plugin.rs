use crate::GameActivity;
use bevy::prelude::{App, IntoScheduleConfigs, OnEnter, OnExit, Plugin, Update, in_state};

use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_components::*;
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_systems::*;
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_ui_components::*;
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_ui_systems::*;

pub struct ShedCommodityCardsPlugin;

impl Plugin for ShedCommodityCardsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NeedsToShedCommodityCards>()
            .register_type::<AwaitingCommodityShedSelection>()
            .init_resource::<CommodityShedSelectionState>()
            .add_systems(
                OnEnter(GameActivity::ShedCommodityCards),
                start_shed_commodity_cards,
            )
            .add_systems(
                Update,
                (
                    apply_commodity_shed,
                    spawn_commodity_shed_ui,
                    update_commodity_shed_ui,
                    cleanup_commodity_shed_ui,
                )
                    .run_if(in_state(GameActivity::ShedCommodityCards)),
            )
            .add_systems(
                OnExit(GameActivity::ShedCommodityCards),
                cleanup_commodity_shed_ui_on_exit,
            );
    }
}
