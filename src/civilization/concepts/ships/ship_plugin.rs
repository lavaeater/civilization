use crate::civilization::components::{OpenSea, SeaPassage};
use crate::civilization::concepts::ships::ship_components::{PlayerShips, Ship, ShipStock};
use crate::civilization::concepts::ships::ship_systems::{
    advance_ship_construction, enter_ship_construction,
};
use crate::civilization::concepts::ships::ship_ui_components::{
    AwaitingShipPlacement, ShipConstructionState,
};
use crate::civilization::concepts::ships::ship_ui_systems::{
    despawn_ship_construction_ui, spawn_ship_construction_ui, update_ship_construction_ui,
};
use crate::GameActivity;
use bevy::app::{App, Plugin};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, Update};

pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ship>()
            .register_type::<ShipStock>()
            .register_type::<PlayerShips>()
            .register_type::<SeaPassage>()
            .register_type::<OpenSea>()
            .register_type::<AwaitingShipPlacement>()
            .init_resource::<ShipConstructionState>()
            .add_systems(OnEnter(GameActivity::ShipConstruction), enter_ship_construction)
            .add_systems(
                Update,
                (
                    spawn_ship_construction_ui,
                    update_ship_construction_ui,
                    despawn_ship_construction_ui,
                    advance_ship_construction,
                )
                    .run_if(in_state(GameActivity::ShipConstruction)),
            );
    }
}
