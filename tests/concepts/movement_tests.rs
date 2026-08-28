use crate::{create_area, create_area_w_components, setup_bevy_app, setup_player};
use adv_civ::civilization::{
    AvailableMoves, CameraFocusQueue, GameArea, GameFaction, GameMove, LandPassage,
    MoveTokenFromAreaToAreaCommand, PlayerAreas, PlayerMovementEnded, PlayerShips, Population,
    RecalculatePlayerMoves, SeaPassage, ShipFerryCommand, TokenHasMoved, TokenStock,
    execute_ship_ferry, move_tokens_from_area_to_area, recalculate_movement_moves_for_player,
};
use adv_civ::{GameActivity, GameState};
use bevy::app::Update;
use bevy::prelude::{App, AppExtStates, Messages, Name, Transform};
use bevy::state::app::StatesPlugin;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_message::<MoveTokenFromAreaToAreaCommand>()
        .add_message::<RecalculatePlayerMoves>()
        .init_resource::<CameraFocusQueue>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, move_tokens_from_area_to_area);
    app
}

#[test]
fn moved_tokens_get_token_has_moved_component_added() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(6);

    for token in player_one_tokens.drain(0..3).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }

    let from_area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let to_area = app
        .world_mut()
        .spawn((
            Name::new("crete"),
            GameArea::new(2),
            LandPassage::default(),
            Population::new(3),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    let mut events = app
        .world_mut()
        .resource_mut::<Messages<MoveTokenFromAreaToAreaCommand>>();

    let _ = events.write(MoveTokenFromAreaToAreaCommand::new(
        from_area, to_area, 2, player_one,
    ));

    // Act
    app.update();
    // Assert
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(player_area.contains(to_area));
    let tokens_in_area = player_area.tokens_for_area(to_area);
    assert!(tokens_in_area.is_some());
    let tokens_in_area = tokens_in_area.unwrap();
    assert_eq!(2, tokens_in_area.len());
    for token in tokens_in_area {
        assert!(app.world().entity(token).get::<TokenHasMoved>().is_some());
    }

    let from_pop = app.world().entity(from_area).get::<Population>();

    let from_tokens = from_pop.unwrap().tokens_for_player(&player_one);
    assert!(from_tokens.is_some());
    let from_tokens = from_tokens.unwrap();
    assert_eq!(1, from_tokens.len());
}

#[test]
fn moving_token_to_area_adds_area_to_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    for token in player_one_tokens.drain(0..3).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }

    let from_area = create_area_w_components(&mut app, "egypt", Some(population));

    let to_area = create_area_w_components(&mut app, "crete", None::<()>);

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<MoveTokenFromAreaToAreaCommand>>();

    events.write(MoveTokenFromAreaToAreaCommand::new(
        from_area, to_area, 2, player_one,
    ));

    // Act
    app.update();
    // Assert
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(player_area.contains(to_area));
}

#[test]
fn moving_all_tokens_from_area_removes_area_from_player_areas() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(4);

    for token in player_one_tokens.drain(0..3).collect::<Vec<_>>() {
        population.add_token_to_area(player_one, token);
    }

    let from_area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
        ))
        .id();

    let to_area = app
        .world_mut()
        .spawn((
            Name::new("crete"),
            GameArea::new(2),
            LandPassage::default(),
            Population::new(3),
        ))
        .id();
    let mut events = app
        .world_mut()
        .resource_mut::<Messages<MoveTokenFromAreaToAreaCommand>>();

    events.write(MoveTokenFromAreaToAreaCommand::new(
        from_area, to_area, 3, player_one,
    ));

    // Act
    app.update();
    // Assert
    let player_area = app.world().entity(player_one).get::<PlayerAreas>();
    assert!(player_area.is_some());
    let player_area = player_area.unwrap();
    assert!(!player_area.contains(from_area));
}

#[test]
fn calculate_one_move() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_message::<RecalculatePlayerMoves>()
            .add_message::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player);
        app
    });

    let area_one = create_area(&mut app, "Egypt", 1);
    let area_two = create_area(&mut app, "Thrace", 2);
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    app.world_mut().entity_mut(area_one).insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

    let mut population = Population::new(4);
    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    app.world_mut().entity_mut(area_one).insert(population);

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));

    app.world_mut()
        .entity_mut(area_two)
        .insert(Population::new(3));

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<RecalculatePlayerMoves>>();

    events.write(RecalculatePlayerMoves::new(player));

    // Act
    app.update();
    // Assert
    let player_moves = app.world().entity(player).get::<AvailableMoves>();

    assert!(player_moves.is_some());
    let player_moves = player_moves.unwrap();
    assert_eq!(player_moves.moves.len(), 2);
    let first_move = player_moves.moves.get(&1).unwrap();
    assert!(matches!(first_move, GameMove::Movement(..)));
    if let GameMove::Movement(m) = first_move {
        assert_eq!(m.max_tokens, 1);
        assert_eq!(m.target, area_two);
        assert_eq!(m.source, area_one);
    }

    let last_move = player_moves.moves.get(&2).unwrap();
    assert!(matches!(last_move, GameMove::EndMovement));
}

#[test]
fn calculate_two_moves() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_message::<RecalculatePlayerMoves>()
            .add_message::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player);
        app
    });

    let area_one = create_area(&mut app, "Egypt", 1);
    let area_two = create_area(&mut app, "Thrace", 2);
    let area_three = create_area(&mut app, "Throgdor", 3);
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    land_passage.add_passage(area_three);
    app.world_mut().entity_mut(area_one).insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

    let mut population = Population::new(4);
    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    app.world_mut().entity_mut(area_one).insert(population);

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));

    app.world_mut()
        .entity_mut(area_two)
        .insert(Population::new(3));

    app.world_mut()
        .entity_mut(area_three)
        .insert(Population::new(2));

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<RecalculatePlayerMoves>>();

    events.write(RecalculatePlayerMoves::new(player));

    // Act
    app.update();
    // Assert
    let player_moves = app.world().entity(player).get::<AvailableMoves>();

    assert!(player_moves.is_some());
    let player_moves = player_moves.unwrap();
    assert_eq!(player_moves.moves.len(), 3);
    let first_move = player_moves.moves.get(&1).unwrap();
    assert!(matches!(first_move, GameMove::Movement(..)));
    if let GameMove::Movement(m) = first_move {
        assert_eq!(m.max_tokens, 1);
        assert_eq!(m.target, area_two);
        assert_eq!(m.source, area_one);
    }

    let second_move = player_moves.moves.get(&2).unwrap();
    assert!(matches!(second_move, GameMove::Movement(..)));
    if let GameMove::Movement(m) = second_move {
        assert_eq!(m.max_tokens, 1);
        assert_eq!(m.target, area_three);
        assert_eq!(m.source, area_one);
    }

    let last_move = player_moves.moves.get(&3).unwrap();
    assert!(matches!(last_move, GameMove::EndMovement));
}

#[test]
fn calculate_moves_after_having_moved() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_message::<RecalculatePlayerMoves>()
            .add_message::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player);
        app
    });

    let area_one = create_area(&mut app, "Egypt", 1);
    let area_two = create_area(&mut app, "Thrace", 2);
    let area_three = create_area(&mut app, "Throgdor", 3);
    let mut land_passage = LandPassage::default();
    land_passage.add_passage(area_two);
    land_passage.add_passage(area_three);
    app.world_mut().entity_mut(area_one).insert(land_passage);
    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = TokenStock::new(47, tokens.drain(0..4).collect());

    let mut population = Population::new(4);
    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    let token = stock.remove_token_from_stock().unwrap();
    player_areas.add_token_to_area(area_one, token);
    population.add_token_to_area(player, token);

    app.world_mut().entity_mut(area_one).insert(population);

    app.world_mut()
        .entity_mut(area_two)
        .insert(Population::new(3));

    app.world_mut()
        .entity_mut(area_three)
        .insert(Population::new(2));

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<RecalculatePlayerMoves>>();

    events.write(RecalculatePlayerMoves::new(player));

    // Act
    app.update();
    // Assert
    let player_moves = app.world().entity(player).get::<AvailableMoves>();

    assert!(player_moves.is_some());
    let player_moves = player_moves.unwrap();
    assert_eq!(player_moves.moves.len(), 3);
    let first_move = player_moves.moves.get(&1).unwrap();
    assert!(matches!(first_move, GameMove::Movement(..)));
    if let GameMove::Movement(m) = first_move {
        assert_eq!(m.max_tokens, 3);
        assert_eq!(m.target, area_two);
        assert_eq!(m.source, area_one);
    }

    let second_move = player_moves.moves.get(&2).unwrap();
    assert!(matches!(second_move, GameMove::Movement(..)));
    if let GameMove::Movement(m) = second_move {
        assert_eq!(m.max_tokens, 3);
        assert_eq!(m.target, area_three);
        assert_eq!(m.source, area_one);
    }

    let last_move = player_moves.moves.get(&3).unwrap();
    assert!(matches!(last_move, GameMove::EndMovement));
}

/// Rule 23.1: a ship with no tokens of its own to carry -- and sitting in an
/// area where the player has no population at all -- must still generate a
/// move for itself. Previously ship-move generation piggybacked entirely on
/// the token-presence loop, so a stranded empty ship had no move at all.
#[test]
fn calculate_moves_generates_a_ship_move_for_an_empty_ship_with_no_tokens_in_its_area() {
    // Arrange
    let mut app = setup_bevy_app(|mut app| {
        app.add_message::<RecalculatePlayerMoves>()
            .add_message::<PlayerMovementEnded>()
            .add_systems(Update, recalculate_movement_moves_for_player);
        app
    });

    let source_area = create_area(&mut app, "Crete", 1);
    let target_area = create_area(&mut app, "Aegean Sea", 2);
    let mut sea_passage = SeaPassage::default();
    sea_passage.add_passage(target_area);
    app.world_mut().entity_mut(source_area).insert(sea_passage);
    app.world_mut()
        .entity_mut(source_area)
        .insert(Population::new(4));
    app.world_mut()
        .entity_mut(target_area)
        .insert(Population::new(4));

    let (player, _tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Crete);

    // Player has NO tokens/population anywhere -- only a ship, sitting alone
    // in source_area.
    let mut player_ships = PlayerShips::default();
    let ship = app.world_mut().spawn(Name::new("Ship")).id();
    player_ships.place_ship(source_area, ship);
    app.world_mut()
        .entity_mut(player)
        .insert((PlayerAreas::default(), player_ships));

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.write(RecalculatePlayerMoves::new(player));

    // Act
    app.update();

    // Assert: a ShipFerry move exists carrying 0 tokens, plus EndMovement.
    let player_moves = app.world().entity(player).get::<AvailableMoves>().unwrap();
    let ship_move = player_moves
        .moves
        .values()
        .find_map(|m| match m {
            GameMove::ShipFerry(m) => Some(m),
            _ => None,
        })
        .expect("expected a ShipFerry move for the empty ship");
    assert_eq!(ship_move.source, source_area);
    assert_eq!(ship_move.target, target_area);
    assert_eq!(ship_move.max_tokens, 0);
}

// ── Rule 23.52: ship ferrying ────────────────────────────────────────────────

#[test]
fn ship_ferry_moves_tokens_and_the_ship_to_the_target_area() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_message::<ShipFerryCommand>()
        .add_message::<RecalculatePlayerMoves>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, execute_ship_ferry);

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut source_population = Population::new(6);
    let tokens_to_ferry: Vec<_> = player_one_tokens.drain(0..3).collect();
    for &token in &tokens_to_ferry {
        source_population.add_token_to_area(player_one, token);
    }

    let source_area = create_area_w_components(&mut app, "source", Some(source_population));
    let target_area = create_area_w_components(&mut app, "target", Some(Population::new(6)));

    let mut player_areas = PlayerAreas::default();
    for &token in &tokens_to_ferry {
        player_areas.add_token_to_area(source_area, token);
    }
    app.world_mut().entity_mut(player_one).insert(player_areas);

    let ship = app.world_mut().spawn(Name::new("Ship")).id();
    let mut player_ships = PlayerShips::default();
    player_ships.place_ship(source_area, ship);
    app.world_mut().entity_mut(player_one).insert(player_ships);

    app.world_mut()
        .resource_mut::<Messages<ShipFerryCommand>>()
        .write(ShipFerryCommand::new(
            source_area,
            target_area,
            3,
            player_one,
        ));

    // Act
    app.update();

    // Assert: all 3 tokens moved from source to target population...
    let source_pop = app.world().get::<Population>(source_area).unwrap();
    let target_pop = app.world().get::<Population>(target_area).unwrap();
    assert_eq!(source_pop.population_for_player(player_one), 0);
    assert_eq!(target_pop.population_for_player(player_one), 3);
    for &token in &tokens_to_ferry {
        assert!(app.world().get::<TokenHasMoved>(token).is_some());
    }

    // ...PlayerAreas reflects the move...
    let areas = app.world().get::<PlayerAreas>(player_one).unwrap();
    assert!(!areas.areas().contains(&source_area));
    assert!(areas.areas().contains(&target_area));

    // ...and the ship follows to the target area.
    let ships = app.world().get::<PlayerShips>(player_one).unwrap();
    assert_eq!(ships.ships_in_area(source_area), []);
    assert_eq!(ships.ships_in_area(target_area), &[ship]);
}

#[test]
fn ship_ferry_only_moves_tokens_that_have_not_already_moved() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_message::<ShipFerryCommand>()
        .add_message::<RecalculatePlayerMoves>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, execute_ship_ferry);

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut source_population = Population::new(6);
    let already_moved = player_one_tokens.remove(0);
    let unmoved = player_one_tokens.remove(0);
    source_population.add_token_to_area(player_one, already_moved);
    source_population.add_token_to_area(player_one, unmoved);

    let source_area = create_area_w_components(&mut app, "source", Some(source_population));
    let target_area = create_area_w_components(&mut app, "target", Some(Population::new(6)));

    app.world_mut()
        .entity_mut(already_moved)
        .insert(TokenHasMoved);

    let mut player_areas = PlayerAreas::default();
    player_areas.add_token_to_area(source_area, already_moved);
    player_areas.add_token_to_area(source_area, unmoved);
    app.world_mut().entity_mut(player_one).insert(player_areas);

    let ship = app.world_mut().spawn(Name::new("Ship")).id();
    let mut player_ships = PlayerShips::default();
    player_ships.place_ship(source_area, ship);
    app.world_mut().entity_mut(player_one).insert(player_ships);

    // Ask to ferry up to 5 -- only the 1 unmoved token is eligible.
    app.world_mut()
        .resource_mut::<Messages<ShipFerryCommand>>()
        .write(ShipFerryCommand::new(
            source_area,
            target_area,
            5,
            player_one,
        ));

    // Act
    app.update();

    // Assert: only the unmoved token crossed; the already-moved one stayed put.
    let source_pop = app.world().get::<Population>(source_area).unwrap();
    let target_pop = app.world().get::<Population>(target_area).unwrap();
    assert_eq!(source_pop.population_for_player(player_one), 1);
    assert_eq!(target_pop.population_for_player(player_one), 1);
}

// ── Token conservation: a failed target-side lookup must not drop tokens ───

/// `move_tokens_from_area_to_area` used to remove tokens from the source
/// area's `Population` unconditionally, then only add them to the target
/// inside a nested lookup that could fail independently (e.g. the target
/// area entity has no `Population` component). A failed lookup silently
/// dropped the tokens: not in the source, not in the target, not in stock.
/// This asserts the tokens stay put when the target-side move can't complete.
#[test]
fn move_tokens_from_area_to_area_keeps_tokens_in_source_if_target_lookup_fails() {
    // Arrange
    let mut app = setup_app();

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut population = Population::new(6);
    let moved_tokens: Vec<_> = player_one_tokens.drain(0..3).collect();
    for &token in &moved_tokens {
        population.add_token_to_area(player_one, token);
    }

    let from_area = app
        .world_mut()
        .spawn((
            Name::new("egypt"),
            GameArea::new(1),
            LandPassage::default(),
            population,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Target area intentionally has NO `Population` component -- simulates a
    // stale/invalid move command whose target-side lookup fails.
    let to_area = app
        .world_mut()
        .spawn((Name::new("crete"), GameArea::new(2), LandPassage::default()))
        .id();

    let mut player_areas = PlayerAreas::default();
    for &token in &moved_tokens {
        player_areas.add_token_to_area(from_area, token);
    }
    app.world_mut().entity_mut(player_one).insert(player_areas);

    let mut events = app
        .world_mut()
        .resource_mut::<Messages<MoveTokenFromAreaToAreaCommand>>();
    events.write(MoveTokenFromAreaToAreaCommand::new(
        from_area, to_area, 2, player_one,
    ));

    // Act
    app.update();

    // Assert: nothing vanished -- all 3 tokens are still in the source
    // Population, and PlayerAreas still points only at the source area.
    let from_pop = app.world().entity(from_area).get::<Population>().unwrap();
    assert_eq!(from_pop.population_for_player(player_one), 3);

    let player_areas = app.world().entity(player_one).get::<PlayerAreas>().unwrap();
    assert!(player_areas.contains(from_area));
    assert!(!player_areas.contains(to_area));
}

/// Same conservation guarantee for `execute_ship_ferry`: a target area with no
/// `Population` component must not cause ferried tokens to be removed from
/// the source without ever landing anywhere.
#[test]
fn ship_ferry_keeps_tokens_in_source_if_target_lookup_fails() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_message::<ShipFerryCommand>()
        .add_message::<RecalculatePlayerMoves>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, execute_ship_ferry);

    let (player_one, mut player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    let mut source_population = Population::new(6);
    let tokens_to_ferry: Vec<_> = player_one_tokens.drain(0..3).collect();
    for &token in &tokens_to_ferry {
        source_population.add_token_to_area(player_one, token);
    }

    let source_area = create_area_w_components(&mut app, "source", Some(source_population));
    // Target area has no `Population` component.
    let target_area = app
        .world_mut()
        .spawn((
            Name::new("target"),
            GameArea::new(2),
            LandPassage::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let mut player_areas = PlayerAreas::default();
    for &token in &tokens_to_ferry {
        player_areas.add_token_to_area(source_area, token);
    }
    app.world_mut().entity_mut(player_one).insert(player_areas);

    let ship = app.world_mut().spawn(Name::new("Ship")).id();
    let mut player_ships = PlayerShips::default();
    player_ships.place_ship(source_area, ship);
    app.world_mut().entity_mut(player_one).insert(player_ships);

    app.world_mut()
        .resource_mut::<Messages<ShipFerryCommand>>()
        .write(ShipFerryCommand::new(
            source_area,
            target_area,
            3,
            player_one,
        ));

    // Act
    app.update();

    // Assert: the tokens are still in the source population, not dropped.
    let source_pop = app.world().get::<Population>(source_area).unwrap();
    assert_eq!(source_pop.population_for_player(player_one), 3);

    let areas = app.world().get::<PlayerAreas>(player_one).unwrap();
    assert!(areas.areas().contains(&source_area));
}

/// Rule 23.1: ships move independently of tokens -- an empty ship in an area
/// with no unmoved (or no) tokens must still be able to relocate on its own.
#[test]
fn ship_ferry_moves_an_empty_ship_with_no_tokens_along() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_message::<ShipFerryCommand>()
        .add_message::<RecalculatePlayerMoves>()
        .insert_state(GameState::Playing)
        .add_sub_state::<GameActivity>()
        .add_systems(Update, execute_ship_ferry);

    let (player_one, _player_one_tokens, _) =
        setup_player(&mut app, "player one", GameFaction::Egypt);

    // Source area has no tokens belonging to player_one at all.
    let source_area = create_area_w_components(&mut app, "source", Some(Population::new(6)));
    let target_area = create_area_w_components(&mut app, "target", Some(Population::new(6)));

    app.world_mut()
        .entity_mut(player_one)
        .insert(PlayerAreas::default());

    let ship = app.world_mut().spawn(Name::new("Ship")).id();
    let mut player_ships = PlayerShips::default();
    player_ships.place_ship(source_area, ship);
    app.world_mut().entity_mut(player_one).insert(player_ships);

    app.world_mut()
        .resource_mut::<Messages<ShipFerryCommand>>()
        .write(ShipFerryCommand::new(
            source_area,
            target_area,
            0,
            player_one,
        ));

    // Act
    app.update();

    // Assert: the ship relocated on its own, no tokens involved.
    let ships = app.world().get::<PlayerShips>(player_one).unwrap();
    assert_eq!(ships.ships_in_area(source_area), []);
    assert_eq!(ships.ships_in_area(target_area), &[ship]);
}
