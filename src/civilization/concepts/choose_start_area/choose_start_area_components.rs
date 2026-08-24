use bevy::prelude::{Component, Entity, Resource};

/// A human player's home faction has more than one eligible start area on
/// this map, so they must pick which one to begin in. Attached during
/// `start_game`; removed by `apply_start_area_choice` once resolved.
#[derive(Component)]
pub struct NeedsToChooseStartArea {
    pub eligible: Vec<Entity>,
}

/// Marks the human as currently choosing a start area; removed by
/// `handle_start_area_click` the moment a valid area is clicked.
#[derive(Component, Default)]
pub struct AwaitingStartAreaSelection;

#[derive(Resource, Default)]
pub struct StartAreaSelectionState {
    pub player: Option<Entity>,
    pub eligible: Vec<Entity>,
    pub chosen: Option<Entity>,
}
