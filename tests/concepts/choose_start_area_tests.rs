use adv_civ::GameActivity;
use adv_civ::civilization::{
    AwaitingStartAreaSelection, DebugOptions, Faction, GameArea, GameFaction,
    MoveTokensFromStockToAreaCommand, NeedsToChooseStartArea, StartArea, StartAreaSelectionState,
    apply_start_area_choice, start_game,
};
use adv_civ::player::Player;
use adv_civ::stupid_ai::IsHuman;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Entity, MessageReader, Messages, Name, NextState, Transform, World};

fn base_world() -> World {
    let mut world = World::new();
    world.init_resource::<Messages<MoveTokensFromStockToAreaCommand>>();
    world.init_resource::<NextState<GameActivity>>();
    world.init_resource::<DebugOptions>();
    world.init_resource::<StartAreaSelectionState>();
    world
}

fn spawn_area(world: &mut World, id: i32, faction: Option<GameFaction>) -> Entity {
    let mut entity = world.spawn((
        Name::new(format!("area-{id}")),
        GameArea::new(id),
        Transform::from_xyz(id as f32 * 10.0, 0.0, 0.0),
    ));
    if let Some(faction) = faction {
        entity.insert(StartArea::new(faction));
    }
    entity.id()
}

fn spawn_player(world: &mut World, name: &str, faction: GameFaction, is_human: bool) -> Entity {
    let mut entity = world.spawn((Player, Name::new(name.to_string()), Faction { faction }));
    if is_human {
        entity.insert(IsHuman);
    }
    entity.id()
}

/// Drains the pending `MoveTokensFromStockToAreaCommand` messages into owned
/// (area, player) pairs, so tests can assert on them without needing the
/// (non-`Clone`) message type itself.
fn drain_move_commands(world: &mut World) -> Vec<(Entity, Entity)> {
    fn collect(mut reader: MessageReader<MoveTokensFromStockToAreaCommand>) -> Vec<(Entity, Entity)> {
        reader
            .read()
            .map(|cmd| (cmd.area_entity, cmd.player_entity))
            .collect()
    }
    world.run_system_once(collect).unwrap()
}

/// AI with a single eligible start area places a token there immediately --
/// no selection UI, matches pre-existing behavior.
#[test]
fn ai_with_one_start_area_places_immediately() {
    let mut world = base_world();
    let area = spawn_area(&mut world, 1, Some(GameFaction::Thrace));
    let player = spawn_player(&mut world, "AI Thrace", GameFaction::Thrace, false);

    world.run_system_once(start_game).unwrap();
    world.flush();

    assert!(world.get::<NeedsToChooseStartArea>(player).is_none());
    let placed = drain_move_commands(&mut world);
    assert_eq!(placed, vec![(area, player)]);

    let state = world.resource::<NextState<GameActivity>>();
    assert!(matches!(
        state,
        NextState::Pending(GameActivity::PopulationExpansion)
    ));
}

/// AI with several eligible start areas picks one of them at random --
/// still immediate, no human-style selection gate.
#[test]
fn ai_with_multiple_start_areas_picks_one_at_random() {
    let mut world = base_world();
    let areas: Vec<Entity> = (1..=4)
        .map(|id| spawn_area(&mut world, id, Some(GameFaction::Africa)))
        .collect();
    let player = spawn_player(&mut world, "AI Africa", GameFaction::Africa, false);

    world.run_system_once(start_game).unwrap();
    world.flush();

    assert!(world.get::<NeedsToChooseStartArea>(player).is_none());
    let placed = drain_move_commands(&mut world);
    assert_eq!(placed.len(), 1);
    assert!(areas.contains(&placed[0].0));
    assert_eq!(placed[0].1, player);
}

/// A human faction with multiple eligible start areas must NOT get a token
/// auto-placed -- this is the regression guard for the Egypt/Africa
/// area-collision bug: `start_game` must gate behind a real choice instead
/// of silently picking one for the player.
#[test]
fn human_with_multiple_start_areas_is_gated_behind_selection() {
    let mut world = base_world();
    let areas: Vec<Entity> = (1..=6)
        .map(|id| spawn_area(&mut world, id, Some(GameFaction::Egypt)))
        .collect();
    let player = spawn_player(&mut world, "Human Egypt", GameFaction::Egypt, true);

    world.run_system_once(start_game).unwrap();
    world.flush();

    let needs = world
        .get::<NeedsToChooseStartArea>(player)
        .expect("human must be gated behind a real choice");
    assert_eq!(
        needs.eligible.len(),
        6,
        "all 6 of the faction's areas must be offered"
    );
    for area in &areas {
        assert!(needs.eligible.contains(area));
    }

    assert!(
        drain_move_commands(&mut world).is_empty(),
        "no token should be placed before the human chooses"
    );

    // The phase must not advance while a choice is pending.
    let state = world.resource::<NextState<GameActivity>>();
    assert!(matches!(state, NextState::Unchanged));

    world.run_system_once(apply_start_area_choice).unwrap();
    world.flush();
    assert!(
        world.get::<AwaitingStartAreaSelection>(player).is_some(),
        "the UI gate must open once start_game has handed off"
    );
}

/// A human faction with exactly one eligible start area places immediately,
/// same as AI -- no pointless one-option selection UI.
#[test]
fn human_with_one_start_area_places_immediately() {
    let mut world = base_world();
    let area = spawn_area(&mut world, 1, Some(GameFaction::Thrace));
    let player = spawn_player(&mut world, "Human Thrace", GameFaction::Thrace, true);

    world.run_system_once(start_game).unwrap();
    world.flush();

    assert!(world.get::<NeedsToChooseStartArea>(player).is_none());
    let placed = drain_move_commands(&mut world);
    assert_eq!(placed, vec![(area, player)]);
}

/// End-to-end: once the human's choice is recorded (simulating a click),
/// `apply_start_area_choice` places the token in exactly the chosen area
/// and lets the phase advance.
#[test]
fn confirmed_human_choice_places_token_and_advances_phase() {
    let mut world = base_world();
    let areas: Vec<Entity> = (1..=3)
        .map(|id| spawn_area(&mut world, id, Some(GameFaction::Egypt)))
        .collect();
    let player = spawn_player(&mut world, "Human Egypt", GameFaction::Egypt, true);
    let chosen_area = areas[1];

    world.run_system_once(start_game).unwrap();
    world.flush();
    world.run_system_once(apply_start_area_choice).unwrap();
    world.flush();

    // Simulate the human clicking `chosen_area`.
    world.resource_mut::<StartAreaSelectionState>().chosen = Some(chosen_area);
    world
        .entity_mut(player)
        .remove::<AwaitingStartAreaSelection>();

    world.run_system_once(apply_start_area_choice).unwrap();
    world.flush();

    assert!(world.get::<NeedsToChooseStartArea>(player).is_none());
    assert!(world.get::<AwaitingStartAreaSelection>(player).is_none());

    let placed = drain_move_commands(&mut world);
    assert_eq!(placed, vec![(chosen_area, player)]);

    // No one is left choosing, so the phase can now advance.
    world.run_system_once(apply_start_area_choice).unwrap();
    let state = world.resource::<NextState<GameActivity>>();
    assert!(matches!(
        state,
        NextState::Pending(GameActivity::PopulationExpansion)
    ));
}

/// Regression guard for the original bug report: the debug
/// `human_starting_areas` testing option must never place the human on an
/// area that's already another faction's declared start area.
#[test]
fn debug_starting_areas_never_collide_with_a_claimed_start_area() {
    let mut world = base_world();
    let africa_area = spawn_area(&mut world, 1, Some(GameFaction::Africa));
    let free_areas: Vec<Entity> = (2..=5).map(|id| spawn_area(&mut world, id, None)).collect();
    let player = spawn_player(&mut world, "Human Egypt", GameFaction::Egypt, true);

    world.resource_mut::<DebugOptions>().human_starting_areas = Some(3);

    world.run_system_once(start_game).unwrap();
    world.flush();

    let placed = drain_move_commands(&mut world);
    assert_eq!(placed.len(), 3);
    for (area_entity, player_entity) in &placed {
        assert_ne!(
            *area_entity, africa_area,
            "debug placement must never land on another faction's start area"
        );
        assert!(free_areas.contains(area_entity));
        assert_eq!(*player_entity, player);
    }
}
