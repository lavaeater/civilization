use crate::civilization::concepts::population_expansion::population_expansion_components::ExpandManually;
use crate::civilization::concepts::trade::trade_components::NeedsTradeMove;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use bevy::prelude::{Commands, MessageWriter, Add, On};
use crate::civilization::concepts::check_city_support::check_city_support_components::HasTooManyCities;
use crate::civilization::concepts::city_construction::city_construction_components::IsBuilding;
use crate::civilization::concepts::movement::movement_components::{HasJustMoved, PerformingMovement};

pub fn on_add_manual_expansion(
    trigger: On<Add, ExpandManually>,
    mut event_writer: MessageWriter<RecalculatePlayerMoves>,
) {
    // //debug!("on_add_manual_expansion");
    event_writer.write(RecalculatePlayerMoves::new(trigger.event().entity));
}

pub fn on_add_perform_movement(
    trigger: On<Add, PerformingMovement>,
    mut event_writer: MessageWriter<RecalculatePlayerMoves>,
) {
    // //debug!("on_add_perform_movement");
    event_writer.write(RecalculatePlayerMoves::new(trigger.event().entity));
}

pub fn on_add_has_just_moved(
    trigger: On<Add, HasJustMoved>,
    mut event_writer: MessageWriter<RecalculatePlayerMoves>,
    mut commands: Commands,
) {
    // //debug!("on_add_has_just_moved");
    commands.entity(trigger.event().entity).remove::<HasJustMoved>();
    event_writer.write(RecalculatePlayerMoves::new(trigger.event().entity));
}

pub fn on_add_has_too_many_cities(
    trigger: On<Add, HasTooManyCities>,
    mut event_writer: MessageWriter<RecalculatePlayerMoves>,
) {
    event_writer.write(RecalculatePlayerMoves::new(trigger.event().entity));
}

pub fn on_add_is_building(
    trigger: On<Add, IsBuilding>,
    mut event_writer: MessageWriter<RecalculatePlayerMoves>,
) {
    event_writer.write(RecalculatePlayerMoves::new(trigger.event().entity));
}

pub fn on_add_needs_trade_move(
    trigger: On<Add, NeedsTradeMove>,
    mut event_writer: MessageWriter<RecalculatePlayerMoves>,
) {
    event_writer.write(RecalculatePlayerMoves::new(trigger.event().entity));
}
