use crate::civilization::components::{OpenSea, SeaPassage};
use crate::civilization::concepts::ships::ship_components::{PlayerShips, Ship, ShipStock};
use crate::civilization::concepts::ships::ship_systems::enter_ship_construction;
use crate::GameActivity;
use bevy::app::{App, Plugin};
use bevy::prelude::OnEnter;

pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ship>()
            .register_type::<ShipStock>()
            .register_type::<PlayerShips>()
            .register_type::<SeaPassage>()
            .register_type::<OpenSea>()
            .add_systems(OnEnter(GameActivity::ShipConstruction), enter_ship_construction);
    }
}
