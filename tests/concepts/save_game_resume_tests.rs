use adv_civ::GameActivity;
use adv_civ::civilization::{
    DebugOptions, LoadingFromSave, MoveTokensFromStockToAreaCommand, StartAreaSelectionState,
    apply_start_area_choice, start_game,
};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Messages, NextState, World};

fn base_world() -> World {
    let mut world = World::new();
    world.init_resource::<Messages<MoveTokensFromStockToAreaCommand>>();
    world.init_resource::<NextState<GameActivity>>();
    world.init_resource::<DebugOptions>();
    world.init_resource::<StartAreaSelectionState>();
    world
}

fn loading_from_save_at(activity: GameActivity) -> LoadingFromSave {
    LoadingFromSave {
        saved_activity: activity,
        completed_factions: Vec::new(),
        census_order: Vec::new(),
        left_to_move: Vec::new(),
        current_mover: None,
    }
}

/// Regression test for the savegame bug: resuming a save taken mid-`Movement`
/// must land back on `Movement`, not silently reset to `PopulationExpansion`
/// -- which used to duplicate every player's expansion tokens on top of the
/// already-restored board.
#[test]
fn resuming_mid_movement_save_does_not_get_clobbered_back_to_population_expansion() {
    let mut world = base_world();
    world.insert_resource(loading_from_save_at(GameActivity::Movement));

    // start_game (OnEnter(StartGame)) sees LoadingFromSave and queues the
    // saved activity.
    world.run_system_once(start_game).unwrap();
    world.flush();
    assert!(matches!(
        world.resource::<NextState<GameActivity>>(),
        NextState::Pending(GameActivity::Movement)
    ));

    // apply_start_area_choice also runs this frame (still `in_state(StartGame)`
    // since the transition hasn't applied yet); with no one left to choose a
    // start area on a resumed game, it used to unconditionally overwrite
    // NextState with PopulationExpansion. It must now leave the queued
    // Movement transition alone.
    world.run_system_once(apply_start_area_choice).unwrap();
    assert!(
        matches!(
            world.resource::<NextState<GameActivity>>(),
            NextState::Pending(GameActivity::Movement)
        ),
        "resuming into Movement must not be clobbered back to PopulationExpansion"
    );
}

/// Same regression, generalized: whatever activity was saved must survive
/// `apply_start_area_choice` unclobbered, not just `Movement`.
#[test]
fn resuming_any_saved_activity_survives_apply_start_area_choice() {
    let mut world = base_world();
    world.insert_resource(loading_from_save_at(GameActivity::Census));

    world.run_system_once(start_game).unwrap();
    world.flush();
    world.run_system_once(apply_start_area_choice).unwrap();

    assert!(matches!(
        world.resource::<NextState<GameActivity>>(),
        NextState::Pending(GameActivity::Census)
    ));
}

/// Sanity check that the `LoadingFromSave` guard doesn't leak into normal
/// (non-resumed) games: with nobody needing a start-area choice and no
/// `LoadingFromSave` resource present, the "nothing to choose" branch must
/// still advance a fresh game to `PopulationExpansion` as before.
#[test]
fn fresh_game_without_loading_from_save_still_advances_to_population_expansion() {
    let mut world = base_world();

    world.run_system_once(apply_start_area_choice).unwrap();

    assert!(matches!(
        world.resource::<NextState<GameActivity>>(),
        NextState::Pending(GameActivity::PopulationExpansion)
    ));
}
