use bevy::prelude::{debug, Commands, EventWriter, OnAdd, Trigger};
use crate::civilization::city_construction::city_construction_components::IsBuilding;
use crate::civilization::city_support::city_support_components::HasTooManyCities;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::movement::movement_components::{HasJustMoved, PerformingMovement};
use crate::civilization::population_expansion::population_expansion_components::ExpandManually;

pub fn on_add_manual_expansion(
    trigger: Trigger<OnAdd, ExpandManually>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    // debug!("on_add_manual_expansion");
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}

pub fn on_add_perform_movement(
    trigger: Trigger<OnAdd, PerformingMovement>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    // debug!("on_add_perform_movement");
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}

pub fn on_add_has_just_moved(
    trigger: Trigger<OnAdd, HasJustMoved>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
    mut commands: Commands,
) {
    // debug!("on_add_has_just_moved");
    commands.entity(trigger.entity()).remove::<HasJustMoved>();
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}

pub fn on_add_has_too_many_cities(
    trigger: Trigger<OnAdd, HasTooManyCities>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}

pub fn on_add_is_building(
    trigger: Trigger<OnAdd, IsBuilding>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}