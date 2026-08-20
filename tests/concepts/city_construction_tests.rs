use crate::{create_area, setup_bevy_app, setup_player};
use adv_civ::civilization::*;
use bevy::app::Update;
use bevy::prelude::Messages;

/// Regression test: `recalculate_city_construction_moves_for_player` must
/// check the real `Population` component, not `PlayerAreas`'s cached mirror
/// of it -- the two can drift apart (see `log_round_state`'s DESYNC
/// diagnostic), and if the cache under-reports population a player with
/// genuinely enough tokens on the board would silently be denied their
/// build option (and lose the surplus to RemoveSurplusPopulation instead).
#[test]
fn build_move_is_offered_from_real_population_even_if_player_areas_cache_is_stale() {
    let mut app = setup_bevy_app(|mut app| {
        app.add_message::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_city_construction_moves_for_player);
        app
    });

    let (player, tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
    let area = create_area(&mut app, "Hattusa", 1);

    // Real board state: 12 of this player's tokens in the area -- exactly at
    // the no-city-site build threshold.
    let mut population = Population::new(20);
    for token in tokens.iter().take(12) {
        population.add_token_to_area(player, *token);
    }
    app.world_mut().entity_mut(area).insert(population);

    // Stale cache: PlayerAreas only knows about 2 of those tokens in the
    // area, as if earlier token movement updated Population but not this
    // mirror.
    let mut player_areas = PlayerAreas::default();
    for token in tokens.iter().take(2) {
        player_areas.add_token_to_area(area, *token);
    }
    app.world_mut().entity_mut(player).insert(player_areas);

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.write(RecalculatePlayerMoves::new(player));

    app.update();

    let player_moves = app
        .world()
        .entity(player)
        .get::<AvailableMoves>()
        .expect("player should still have a build move despite the stale PlayerAreas cache");
    let has_build_move = player_moves
        .moves
        .values()
        .any(|m| matches!(m, GameMove::CityConstruction(build) if build.target == area));
    assert!(has_build_move, "expected a CityConstruction move for the area");
}
