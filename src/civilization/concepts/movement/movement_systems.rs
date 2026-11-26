use crate::GameActivity;
use crate::civilization::components::{
    Faction, FixTokenPositions, GameArea, PlayerAreas, Population, Token,
};
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::map::CameraFocusQueue;
use crate::civilization::concepts::movement::movement_components::*;
use crate::civilization::concepts::movement::movement_events::*;
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::civilization::game_moves::ship_routes;
use crate::civilization::game_moves::{AvailableMoves, RecalculatePlayerMoves};
use crate::civilization::{
    CivCardName, OpenSea, PlayerCivilizationCards, PlayerShips, SHIP_CAPACITY, SeaPassage,
    ShipVoyages, ship_range,
};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use bevy::prelude::{
    Commands, Entity, Has, MessageReader, MessageWriter, Name, NextState, On, Query, Remove, Res,
    ResMut, Transform, With, Without, info, warn,
};
use bevy::time::Time;

pub fn start_movement_activity(
    mut game_info: ResMut<GameInfoAndStuff>,
    mut next_player: MessageWriter<NextPlayerStarted>,
    loading_from_save: Option<Res<LoadingFromSave>>,
    faction_query: Query<(Entity, &Faction), With<Player>>,
    mut voyages: ResMut<ShipVoyages>,
    mut commands: Commands,
) {
    voyages.voyages.clear();
    if let Some(ref save_state) = loading_from_save {
        // Restore census_order and left_to_move from save data
        // Resolve faction names back to entities
        let faction_to_entity: bevy::platform::collections::HashMap<_, _> =
            faction_query.iter().map(|(e, f)| (f.faction, e)).collect();

        game_info.census_order = save_state
            .census_order
            .iter()
            .filter_map(|f| faction_to_entity.get(f).copied())
            .collect();
        game_info.left_to_move = save_state
            .left_to_move
            .iter()
            .filter_map(|f| faction_to_entity.get(f).copied())
            .collect();

        // Restore the player who was actively moving (already popped from left_to_move)
        if let Some(ref mover_faction) = save_state.current_mover {
            if let Some(&mover_entity) = faction_to_entity.get(mover_faction) {
                info!("[MOVEMENT] Restoring current mover: {:?}", mover_faction);
                commands.entity(mover_entity).insert(PerformingMovement);
            } else {
                info!("[MOVEMENT] No current mover to restore, advancing to next player");
                next_player.write(NextPlayerStarted);
            }
        } else {
            info!("[MOVEMENT] No current mover in save, advancing to next player");
            next_player.write(NextPlayerStarted);
        }

        info!(
            "[MOVEMENT] Restored from save: {} in census_order, {} left to move",
            game_info.census_order.len(),
            game_info.left_to_move.len()
        );

        commands.remove_resource::<LoadingFromSave>();
    } else {
        game_info.left_to_move = game_info.census_order.clone();
        game_info.left_to_move.reverse();
        next_player.write(NextPlayerStarted);
    }
}

pub fn prepare_next_mover(
    mut started: MessageReader<NextPlayerStarted>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    _names: Query<&Name>,
) {
    for _ in started.read() {
        if let Some(to_move) = game_info.left_to_move.pop() {
            commands.entity(to_move).insert(PerformingMovement);
        } else {
            info!("No more players to move, transitioning to Conflict");
            next_state.set(GameActivity::Conflict);
        }
    }
}

pub fn on_exit_movement(
    token_query: Query<Entity, With<TokenHasMoved>>,
    player_query: Query<Entity, With<Player>>,
    mut voyages: ResMut<ShipVoyages>,
    mut commands: Commands,
) {
    voyages.voyages.clear();
    for player in player_query.iter() {
        commands.entity(player).remove::<PerformingMovement>();
    }

    for token in token_query.iter() {
        commands.entity(token).remove::<TokenHasMoved>();
    }
}

pub fn player_end_movement(
    mut end_event: MessageReader<PlayerMovementEnded>,
    mut commands: Commands,
    mut next_player: MessageWriter<NextPlayerStarted>,
    names: Query<&Name>,
    player_ships: Query<&PlayerShips>,
    mut voyages: ResMut<ShipVoyages>,
) {
    for end_movement_event in end_event.read() {
        // Nobody stays aboard past the phase (rule 23.56): they step off
        // where their ship is, which is where they already stand.
        if let Ok(ships) = player_ships.get(end_movement_event.player) {
            for ship in ships.ships_by_area.values().flatten() {
                if let Some(voyage) = voyages.voyages.get_mut(ship) {
                    voyage.cargo.clear();
                }
            }
        }
        let name = names
            .get(end_movement_event.player)
            .map_or("?", bevy::prelude::Name::as_str);
        info!("Player {} has ended movement", name);
        commands
            .entity(end_movement_event.player)
            .remove::<PerformingMovement>();
        commands
            .entity(end_movement_event.player)
            .remove::<AvailableMoves>();
        next_player.write(NextPlayerStarted);
    }
}

pub fn move_tokens_from_area_to_area(
    mut move_events: MessageReader<MoveTokenFromAreaToAreaCommand>,
    mut pop_query: Query<
        (
            &mut Population,
            &Transform,
            Option<&Name>,
            Option<&GameArea>,
        ),
        Without<Token>,
    >,
    mut commands: Commands,
    mut player_areas: Query<&mut PlayerAreas>,
    tokens_that_can_move: Query<&Token, Without<TokenHasMoved>>,
    token_transform: Query<&Transform, With<Token>>,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
    human_query: Query<Entity, With<IsHuman>>,
    player_is_human: Query<Has<IsHuman>>,
    mut camera_focus: ResMut<CameraFocusQueue>,
) {
    let human_player = human_query.iter().next();

    for ev in move_events.read() {
        // Read-only phase, so this borrow of `pop_query` doesn't overlap with
        // the mutable borrows taken below.
        let source_tokens = pop_query
            .get(ev.source_area)
            .ok()
            .and_then(|(pop, _, _, _)| pop.player_tokens().get(&ev.player).cloned());

        if let Some(player_tokens) = source_tokens {
            let tokens_that_can_move = player_tokens
                .iter()
                .filter(|t| tokens_that_can_move.get(**t).is_ok())
                .copied()
                .collect::<Vec<_>>();
            if tokens_that_can_move.len() < ev.number_of_tokens {
                recalculate_player_moves.write(RecalculatePlayerMoves::new(ev.player));
            } else {
                let tokens_to_move = tokens_that_can_move
                    .iter()
                    .take(ev.number_of_tokens)
                    .copied()
                    .collect::<Vec<_>>();

                // Only remove tokens from the source once we've confirmed the
                // target area and the mover's PlayerAreas both exist, so a
                // missing component can't strand tokens between the two --
                // previously the source removal ran unconditionally while the
                // target-side insert was gated, silently dropping tokens on a
                // failed lookup (token-conservation bug).
                let target_ready = pop_query.get(ev.target_area).is_ok();
                let player_area_ready = player_areas.get(ev.player).is_ok();

                if target_ready
                    && player_area_ready
                    && let Ok((mut from_pop, _, _, _)) = pop_query.get_mut(ev.source_area)
                {
                    for token in &tokens_to_move {
                        from_pop.remove_token_from_area(ev.player, *token);
                    }

                    if let Ok((mut to_pop, target_transform, area_name, game_area)) =
                        pop_query.get_mut(ev.target_area)
                    {
                        // Check if AI is moving into an area with human player tokens
                        let mover_is_ai = player_is_human.get(ev.player).map_or(true, |h| !h);
                        let human_has_tokens = human_player.is_some_and(|h| to_pop.has_player(&h));

                        if mover_is_ai && human_has_tokens {
                            let area_desc = area_name
                                .map(std::string::ToString::to_string)
                                .or_else(|| game_area.map(|a| format!("Area {}", a.id)))
                                .unwrap_or_else(|| "unknown area".to_string());
                            camera_focus.add_focus(
                                target_transform.translation,
                                1.0,
                                format!("AI moving into {area_desc}"),
                            );
                        }

                        if let Ok(mut player_area) = player_areas.get_mut(ev.player) {
                            let target_pos = target_transform.translation;
                            for token in &tokens_to_move {
                                commands.entity(*token).insert(TokenHasMoved);
                                if let Ok(token_transform) = token_transform.get(*token) {
                                    let start_pos = token_transform.translation;
                                    commands.entity(*token).insert(TokenMoveAnimation::new(
                                        start_pos, target_pos,
                                        0.15, // 150ms animation duration
                                    ));
                                }
                                player_area.remove_token_from_area(&ev.source_area, *token);
                                to_pop.add_token_to_area(ev.player, *token);
                                player_area.add_token_to_area(ev.target_area, *token);
                            }
                        }
                    }
                }
            }
        }
        commands.entity(ev.player).insert(HasJustMoved);
        commands.entity(ev.source_area).insert(FixTokenPositions);
        commands.entity(ev.target_area).insert(FixTokenPositions);
    }
}

/// Executes one leg of a ship's voyage (rule 23.5): tokens aboard may step
/// off at the source, unmoved tokens there may step aboard (at most five
/// aboard), and the ship sails to the target with everyone still aboard,
/// spending the water areas it enters from its range for the phase.
///
/// Tokens that step aboard are marked `TokenHasMoved` at once: they cannot
/// then move overland or board another ship (rules 23.3, 23.56). Tokens
/// aboard always stand in the ship's current area, so stepping off moves
/// nothing.
pub fn execute_ship_ferry(
    mut ferry_events: MessageReader<ShipFerryCommand>,
    mut pop_query: Query<(&mut Population, &Transform), Without<Token>>,
    mut player_ships_query: Query<(&mut PlayerShips, Option<&PlayerCivilizationCards>)>,
    mut player_areas_query: Query<&mut PlayerAreas>,
    tokens_that_can_move: Query<&Token, Without<TokenHasMoved>>,
    token_transform: Query<&Transform, With<Token>>,
    sea: Query<&SeaPassage>,
    open_sea: Query<Has<OpenSea>>,
    mut voyages: ResMut<ShipVoyages>,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
    mut commands: Commands,
) {
    for ev in ferry_events.read() {
        let Ok((mut ships, civ_cards)) = player_ships_query.get_mut(ev.player) else {
            continue;
        };
        let astronomy = civ_cards.is_some_and(|c| c.owns(&CivCardName::Astronomy));
        let range = ship_range(civ_cards.is_some_and(|c| c.owns(&CivCardName::ClothMaking)));
        let ships_here = ships.ships_in_area(ev.source_area).to_vec();
        let cost = ship_routes(
            ev.source_area,
            voyages.best_range_left(&ships_here, range),
            astronomy,
            &sea,
            &open_sea,
        )
        .into_iter()
        .find(|(area, _)| *area == ev.target_area)
        .map(|(_, cost)| cost);
        let ship = cost.and_then(|cost| match ev.ship {
            Some(ship) if ships_here.contains(&ship) && voyages.areas_left(ship, range) >= cost => {
                Some(ship)
            }
            Some(_) => None,
            None => voyages.pick_ship(&ships_here, range, cost),
        });
        let (Some(cost), Some(ship)) = (cost, ship) else {
            warn!(
                "[SHIPS] No ship at {:?} can reach {:?} this phase",
                ev.source_area, ev.target_area
            );
            recalculate_player_moves.write(RecalculatePlayerMoves::new(ev.player));
            continue;
        };
        let (target_ready, target_pos) = pop_query
            .get(ev.target_area)
            .map_or((false, bevy::math::Vec3::default()), |(_, t)| {
                (true, t.translation)
            });
        if !target_ready || player_areas_query.get(ev.player).is_err() {
            recalculate_player_moves.write(RecalculatePlayerMoves::new(ev.player));
            continue;
        }

        let voyage = voyages.voyages.entry(ship).or_default();
        // Step off first, so the room they leave can be filled here.
        let dropped = ev.drop.min(voyage.cargo.len());
        voyage.cargo.drain(..dropped);
        let room = SHIP_CAPACITY.saturating_sub(voyage.cargo.len());
        let boarding: Vec<Entity> = pop_query
            .get(ev.source_area)
            .ok()
            .and_then(|(pop, _)| pop.player_tokens().get(&ev.player).cloned())
            .map_or_else(Vec::new, |tokens| {
                tokens
                    .into_iter()
                    .filter(|t| tokens_that_can_move.get(*t).is_ok())
                    .take(ev.number_of_tokens.min(room))
                    .collect()
            });
        for &token in &boarding {
            commands.entity(token).insert(TokenHasMoved);
        }
        voyage.cargo.extend(boarding.iter().copied());

        // Everyone aboard sails.
        let aboard = voyage.cargo.clone();
        if let Ok((mut from_pop, _)) = pop_query.get_mut(ev.source_area) {
            for &token in &aboard {
                from_pop.remove_token_from_area(ev.player, token);
            }
        }
        if let Ok((mut to_pop, _)) = pop_query.get_mut(ev.target_area) {
            for &token in &aboard {
                to_pop.add_token_to_area(ev.player, token);
            }
        }
        if let Ok(mut player_areas) = player_areas_query.get_mut(ev.player) {
            for &token in &aboard {
                player_areas.remove_token_from_area(&ev.source_area, token);
                player_areas.add_token_to_area(ev.target_area, token);
                if let Ok(start) = token_transform.get(token) {
                    commands.entity(token).insert(TokenMoveAnimation::new(
                        start.translation,
                        target_pos,
                        0.25, // slightly longer than land movement to feel different
                    ));
                }
            }
        }
        voyage.areas_moved += cost;
        if ev.unload_on_arrival {
            voyage.cargo.clear();
        }
        let left = range.saturating_sub(voyage.areas_moved);

        if ships.take_ship(ev.source_area, ship) {
            ships.place_ship(ev.target_area, ship);
            commands.entity(ship).insert(Transform::from_translation(
                target_pos + bevy::math::Vec3::new(8.0, 8.0, 2.0),
            ));
        }
        info!(
            "[SHIPS] Ship {ship:?} sails {:?} -> {:?} ({cost} area(s), {left} left): {dropped} off, {} on, {} aboard",
            ev.source_area,
            ev.target_area,
            boarding.len(),
            aboard.len(),
        );

        commands.entity(ev.player).insert(HasJustMoved);
        commands.entity(ev.source_area).insert(FixTokenPositions);
        commands.entity(ev.target_area).insert(FixTokenPositions);
    }
}

/// A token that leaves the board drops any slide it was making, so it is not
/// moved back to that old spot once it is placed again.
pub fn drop_animation_when_token_leaves_board(
    trigger: On<Remove<Transform>>,
    animating: Query<(), (With<Token>, With<TokenMoveAnimation>)>,
    mut commands: Commands,
) {
    let token = trigger.event().entity;
    if animating.contains(token) {
        commands.entity(token).try_remove::<TokenMoveAnimation>();
    }
}

/// System to animate tokens moving from one area to another
pub fn animate_token_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TokenMoveAnimation)>,
) {
    for (entity, mut transform, mut animation) in &mut query {
        animation.elapsed += time.delta_secs();
        transform.translation = animation.current_position();

        if animation.is_complete() {
            // Ensure we're exactly at the end position
            transform.translation = animation.end_pos;
            commands.entity(entity).remove::<TokenMoveAnimation>();
        }
    }
}

#[cfg(test)]
mod ship_voyage_tests {
    use super::*;
    use crate::civilization::game_moves::RecalculatePlayerMoves;
    use bevy::prelude::{App, Update};

    /// Four coasts in a row, A - B - C - D - E, and a player with five
    /// tokens in A, two in B and a ship in A.
    fn setup() -> (App, Entity, [Entity; 5]) {
        let mut app = App::new();
        app.add_message::<ShipFerryCommand>()
            .add_message::<RecalculatePlayerMoves>()
            .init_resource::<ShipVoyages>()
            .add_systems(Update, execute_ship_ferry);
        let world = app.world_mut();
        let coasts = [(); 5].map(|()| {
            world
                .spawn((
                    Population::new(10),
                    SeaPassage::default(),
                    Transform::default(),
                ))
                .id()
        });
        for pair in coasts.windows(2) {
            world
                .get_mut::<SeaPassage>(pair[0])
                .unwrap()
                .add_passage(pair[1]);
            world
                .get_mut::<SeaPassage>(pair[1])
                .unwrap()
                .add_passage(pair[0]);
        }
        let player = world.spawn_empty().id();
        let ship = world.spawn_empty().id();
        let mut areas = PlayerAreas::default();
        for (area, n) in [(coasts[0], 5), (coasts[1], 2)] {
            for _ in 0..n {
                let token = world.spawn((Token::new(player), Transform::default())).id();
                world
                    .get_mut::<Population>(area)
                    .unwrap()
                    .add_token_to_area(player, token);
                areas.add_token_to_area(area, token);
            }
        }
        let mut ships = PlayerShips::default();
        ships.place_ship(coasts[0], ship);
        world.entity_mut(player).insert((areas, ships));
        (app, player, coasts)
    }

    fn sail(app: &mut App, command: ShipFerryCommand) {
        app.world_mut().write_message(command);
        app.update();
    }

    fn tokens(app: &App, area: Entity, player: Entity) -> usize {
        app.world()
            .get::<Population>(area)
            .unwrap()
            .population_for_player(player)
    }

    #[test]
    fn tokens_ride_several_legs_stepping_on_and_off_on_the_way() {
        let (mut app, player, [a, b, c, d, far]) = setup();
        // Leg 1: one aboard in A, sail to B.
        sail(&mut app, ShipFerryCommand::leg(a, b, 0, 1, player));
        assert_eq!((tokens(&app, a, player), tokens(&app, b, player)), (4, 3));
        // Leg 2: drop that one in B, take both B tokens on, sail to C.
        sail(&mut app, ShipFerryCommand::leg(b, c, 1, 2, player));
        assert_eq!((tokens(&app, b, player), tokens(&app, c, player)), (1, 2));
        // Leg 3: drop one, sail on with the other. The token dropped in B
        // has moved and cannot board again.
        sail(&mut app, ShipFerryCommand::leg(c, d, 1, 5, player));
        assert_eq!((tokens(&app, c, player), tokens(&app, d, player)), (1, 1));
        // Leg 4 uses the last of the four areas; a fifth is refused.
        sail(&mut app, ShipFerryCommand::leg(d, far, 0, 0, player));
        assert_eq!(tokens(&app, far, player), 1);
        sail(&mut app, ShipFerryCommand::leg(far, d, 0, 0, player));
        assert_eq!(tokens(&app, far, player), 1, "out of range");
        let ships = app.world().get::<PlayerShips>(player).unwrap();
        assert_eq!(ships.ships_in_area(far).len(), 1);
        // Nothing was lost on the way.
        let total: usize = [a, b, c, d, far]
            .iter()
            .map(|x| tokens(&app, *x, player))
            .sum();
        assert_eq!(total, 7);
        let areas = app.world().get::<PlayerAreas>(player).unwrap();
        assert_eq!(areas.total_population(), 7);
    }

    #[test]
    fn a_ship_carries_five_at_most_and_a_one_trip_ferry_unloads() {
        let (mut app, player, [a, b, c, ..]) = setup();
        // Seven would board in B (five from A, two there); five fit.
        sail(&mut app, ShipFerryCommand::leg(a, b, 0, 5, player));
        sail(&mut app, ShipFerryCommand::leg(b, c, 0, 5, player));
        assert_eq!(tokens(&app, c, player), 5);
        assert_eq!(tokens(&app, b, player), 2);
        // The AI's one-trip ferry leaves nobody aboard.
        sail(&mut app, ShipFerryCommand::new(c, b, 0, player));
        let voyages = app.world().resource::<ShipVoyages>();
        assert!(voyages.voyages.values().all(|v| v.cargo.is_empty()));
        assert_eq!(tokens(&app, b, player), 7);
    }
}

#[cfg(test)]
mod slide_tests {
    use super::*;
    use crate::civilization::components::{FixTokenPositions, Population, Token};
    use crate::civilization::general_systems::fix_token_positions;
    use bevy::prelude::{App, IntoScheduleConfigs, Sprite, Update, Vec3, Visibility};

    /// The play-test ghost: a token whose slide was cut short by the end of
    /// Movement went back to stock, was placed elsewhere, and a round later
    /// the old slide finished, drawing it on a city site it had long left.
    #[test]
    fn a_token_leaving_the_board_forgets_its_slide() {
        let mut app = App::new();
        app.add_observer(drop_animation_when_token_leaves_board);
        let player = app.world_mut().spawn_empty().id();
        let token = app
            .world_mut()
            .spawn((
                Token::new(player),
                Sprite::default(),
                Transform::default(),
                TokenMoveAnimation::new(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), 0.15),
            ))
            .id();
        app.world_mut()
            .entity_mut(token)
            .remove::<(Sprite, Transform, Visibility)>();
        app.update();
        assert!(app.world().get::<TokenMoveAnimation>(token).is_none());
    }

    /// A token sliding in ends on its own spot in the area, not on the
    /// centre where a city would stand.
    #[test]
    fn a_sliding_token_lands_on_its_spot() {
        let mut app = App::new();
        app.add_systems(Update, (fix_token_positions, animate_token_movement).chain())
            .init_resource::<Time>();
        let player = app.world_mut().spawn_empty().id();
        let others = app.world_mut().spawn_empty().id();
        // Piles are laid out by player entity; the slider gets the second.
        let (first, second) = if player < others {
            (player, others)
        } else {
            (others, player)
        };
        let centre = Vec3::new(500.0, 500.0, 0.0);
        let token = app
            .world_mut()
            .spawn((
                // The second pile: its spot is off the centre.
                Token::new(second),
                Transform::default(),
                TokenMoveAnimation::new(Vec3::ZERO, centre, 0.0),
            ))
            .id();
        let other = app
            .world_mut()
            .spawn((Token::new(first), Transform::default()))
            .id();
        let mut population = Population::new(3);
        population.add_token_to_area(second, token);
        population.add_token_to_area(first, other);
        app.world_mut().spawn((
            population,
            Transform::from_translation(centre),
            FixTokenPositions,
        ));
        app.update();
        let at = |e| app.world().get::<Transform>(e).unwrap().translation;
        assert!(app.world().get::<TokenMoveAnimation>(token).is_none());
        assert_eq!(at(other), centre);
        assert_eq!(at(token), centre + Vec3::new(15.0, 0.0, 0.0), "not on the city");
    }
}
