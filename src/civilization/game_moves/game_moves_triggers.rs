use bevy::prelude::{EventWriter, OnAdd, Trigger};
use crate::civilization::city_construction::city_construction_components::IsBuilding;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::movement::movement_components::PerformingMovement;
use crate::civilization::population_expansion::population_expansion_components::ExpandManually;

pub fn on_add_manual_expansion(
    trigger: Trigger<OnAdd, ExpandManually>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
}

pub fn on_add_perform_movement(
    trigger: Trigger<OnAdd, PerformingMovement>,
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