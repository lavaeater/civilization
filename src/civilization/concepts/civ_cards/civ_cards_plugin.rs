use crate::GameActivity;
use bevy::prelude::{App, OnEnter, Plugin};

pub struct CivCardsPlugin;

impl Plugin for CivCardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameActivity::AcquireCivilizationCards), check_who_can_buy);
    }
}

fn check_who_can_buy() {
    /*
    To be able to buy, a player must have cards that have a combined value of at least the cost of
    the lowest-valued card they have not yet bought.
    */
}