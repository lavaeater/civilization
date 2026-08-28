use crate::GameActivity;
use crate::civilization::*;
use crate::player::Player;
use crate::stupid_ai::*;
use bevy::math::{Vec2, Vec3, vec3};
use bevy::platform::collections::HashSet;
use bevy::prelude::{
    Bundle, Commands, Entity, KeyCode, MessageReader, MessageWriter, Name, NextState, Query, Res,
    ResMut, SpawnRelated, Sprite, StateTransitionEvent, Transform, With, Without, debug, default,
    info,
};
use bevy_enhanced_input::actions;
use bevy_enhanced_input::prelude::*;
use rand::seq::{IndexedRandom, SliceRandom};

pub const TOKENS_PER_PLAYER: usize = 47;

pub fn start_game(
    mut commands: Commands,
    player_query: Query<(Entity, &Name, &Faction), With<Player>>,
    human_query: Query<Entity, With<IsHuman>>,
    start_area_query: Query<(Entity, &Name, &StartArea)>,
    all_areas_query: Query<Entity, With<GameArea>>,
    mut writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>,
    debug_options: Res<DebugOptions>,
    loading_from_save: Option<Res<LoadingFromSave>>,
) {
    // When loading from save, skip initial token placement (already restored)
    // and transition directly to the saved activity
    if let Some(save_state) = &loading_from_save {
        let activity = save_state.saved_activity.clone();
        info!("Resuming saved game at activity: {:?}", activity);
        next_state.set(activity);
        return;
    }

    debug!("4. Starting the game!");
    let human_entity = human_query.iter().next();

    // Areas already claimed by an existing faction's start_area, excluded
    // from the debug human_starting_areas pool below so a debug placement
    // can never land on top of another faction's home area.
    let claimed_start_areas: bevy::platform::collections::HashSet<Entity> =
        start_area_query.iter().map(|(e, _, _)| e).collect();

    let mut any_pending_choice = false;

    player_query
        .iter()
        .for_each(|(player_entity, name, player_faction)| {
            debug!("Starting the game for player: {:#?}", name);

            // Debug: human player with special starting areas
            let is_human = human_entity == Some(player_entity);
            if is_human && debug_options.human_starting_areas.is_some() {
                #[allow(clippy::unnecessary_unwrap)]
                let num_areas = debug_options.human_starting_areas.unwrap();
                let areas: Vec<Entity> = all_areas_query
                    .iter()
                    .filter(|e| !claimed_start_areas.contains(e))
                    .take(num_areas)
                    .collect();
                for area_entity in areas {
                    debug!(
                        "Debug: Putting a token in area {:?} for human player",
                        area_entity
                    );
                    writer.write(MoveTokensFromStockToAreaCommand {
                        area_entity,
                        player_entity,
                        number_of_tokens: 1,
                    });
                }
                return;
            }

            // Rule: a faction may have more than one eligible home area on
            // this map (e.g. Egypt has 6); the player picks which one to
            // start in. AI just rolls one at random.
            let eligible: Vec<Entity> = start_area_query
                .iter()
                .filter(|(_, _, start_area)| start_area.faction == player_faction.faction)
                .map(|(e, _, _)| e)
                .collect();

            if is_human && eligible.len() > 1 {
                commands
                    .entity(player_entity)
                    .insert(NeedsToChooseStartArea { eligible });
                any_pending_choice = true;
            } else if let Some(&area_entity) = eligible.choose(&mut rand::rng()) {
                writer.write(MoveTokensFromStockToAreaCommand {
                    area_entity,
                    player_entity,
                    number_of_tokens: 1,
                });
            }
        });

    // If a human still needs to pick their start area, `apply_start_area_choice`
    // owns the transition out of `StartGame` once they do.
    if any_pending_choice {
        return;
    }

    // Use debug start activity if set, otherwise normal flow
    let start_activity = debug_options
        .start_at_activity
        .clone()
        .unwrap_or(GameActivity::PopulationExpansion);
    next_state.set(start_activity);
}

pub fn start_game_after_player_setup(mut next_state: ResMut<NextState<GameActivity>>) {
    next_state.set(GameActivity::StartGame);
}

/// Whether a faction should be agent-controlled, from the `AGENT_FACTIONS` env var:
/// `all` (every non-human player) or a comma-separated, case-insensitive list of
/// faction names (e.g. `Egypt,Babylon`). Unset/empty = none.
pub fn faction_is_agent_controlled(faction: GameFaction) -> bool {
    let Ok(val) = std::env::var("AGENT_FACTIONS") else {
        return false;
    };
    let val = val.trim();
    if val.eq_ignore_ascii_case("all") {
        return true;
    }
    val.split(',').any(|name| {
        let name = name.trim();
        !name.is_empty() && format!("{faction:?}").eq_ignore_ascii_case(name)
    })
}

const ANCIENT_RULERS: &[&str] = &[
    "Ramesses",
    "Cleopatra",
    "Nebuchadnezzar",
    "Hammurabi",
    "Cyrus",
    "Darius",
    "Xerxes",
    "Alexander",
    "Ptolemy",
    "Sargon",
    "Ashurbanipal",
    "Hatshepsut",
    "Tutankhamun",
    "Nefertiti",
    "Minos",
    "Agamemnon",
    "Priam",
    "Leonidas",
    "Pericles",
    "Hannibal",
];

pub fn setup_players(
    debug_options: Res<DebugOptions>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    mut commands: Commands,
    mut available_factions: ResMut<AvailableFactions>,
    loading_from_save: Option<Res<LoadingFromSave>>,
) {
    // Skip setup entirely if we're loading from a save file
    if loading_from_save.is_some() {
        info!("Loading from save - skipping setup_players");
        return;
    }

    debug!("3. Setting up players!");
    let mut available_names: Vec<&str> = ANCIENT_RULERS.to_vec();
    available_names.shuffle(&mut rand::rng());

    // Factions reserved for network seats come first, so the multiplayer
    // server knows which factions its seats map to.
    let mut reserved: Vec<GameFaction> = Vec::new();
    for faction in &debug_options.reserved_factions {
        if available_factions.remaining_factions.remove(faction) {
            reserved.push(*faction);
        }
    }

    // If adding a human player, ensure their faction is included first
    let mut factions_to_use: Vec<_> = if debug_options.add_human_player {
        // Start with the human faction
        let mut factions = vec![debug_options.human_faction];
        available_factions
            .remaining_factions
            .remove(&debug_options.human_faction);

        // Add random factions for the remaining players
        let remaining_count = debug_options
            .number_of_players
            .saturating_sub(1 + reserved.len());
        let mut remaining: Vec<_> = available_factions
            .remaining_factions
            .iter()
            .copied()
            .collect();
        remaining.shuffle(&mut rand::rng());
        factions.extend(remaining.into_iter().take(remaining_count));

        // Remove used factions from available
        for f in &factions {
            available_factions.remaining_factions.remove(f);
        }
        factions
    } else {
        // No human player - just pick random factions
        let mut remaining: Vec<_> = available_factions
            .remaining_factions
            .iter()
            .copied()
            .collect();
        remaining.shuffle(&mut rand::rng());
        let factions: Vec<_> = remaining
            .into_iter()
            .take(
                debug_options
                    .number_of_players
                    .saturating_sub(reserved.len()),
            )
            .collect();
        for f in &factions {
            available_factions.remaining_factions.remove(f);
        }
        factions
    };

    factions_to_use.extend(reserved);
    // Shuffle so human isn't always first
    factions_to_use.shuffle(&mut rand::rng());

    for (n, faction) in factions_to_use.into_iter().enumerate() {
        let ruler_name = available_names.pop().unwrap_or("Unknown");
        // Give each AI a personality: a forced one for testing, else round-robin
        // over all archetypes so a full table shows every playstyle.
        let playstyle = debug_options
            .force_playstyle
            .unwrap_or(Playstyle::ALL[n % Playstyle::ALL.len()]);
        info!("[SETUP] {ruler_name} of {faction:?} is playstyle {playstyle:?}");
        // Create Player
        let player = commands
            .spawn((
                Player,
                Name::new(format!("{ruler_name} of {faction:?}")),
                Census { population: 0 },
                Treasury::default(),
                Faction::new(faction),
                PlayerAreas::default(),
                PlayerCities::default(),
                StupidAi,
                Personality::from_playstyle(playstyle),
                PlayerTradeCards::default(),
                PlayerCivilizationCards::default(),
                AstPosition::new(0),
            ))
            .id();

        if debug_options.add_human_player && faction == debug_options.human_faction {
            setup_human_player(
                &debug_options,
                &mut trade_card_resource,
                &mut commands,
                player,
            );
        } else if faction_is_agent_controlled(faction) {
            // Agent-controlled players behave like a remote human (IsHuman → the game
            // waits for them, AI doesn't auto-play) but are tagged AgentControlled so
            // trade/UI logic routes to the agent API instead of a keyboard. See the API.
            commands.entity(player).remove::<StupidAi>();
            commands.entity(player).insert(IsHuman);
            commands.entity(player).insert(AgentControlled);
            info!("Faction {:?} is agent-controlled (AGENT_FACTIONS)", faction);
        }

        // Determine token count - use debug override for human player if set
        let is_human = debug_options.add_human_player && faction == debug_options.human_faction;
        let token_count = if is_human {
            debug_options.human_token_count.unwrap_or(TOKENS_PER_PLAYER)
        } else {
            TOKENS_PER_PLAYER
        };

        let tokens = (0..token_count)
            .map(|_| {
                commands
                    .spawn((Name::new(format!("Token {n}")), Token::new(player)))
                    .id()
            })
            .collect::<Vec<Entity>>();

        let city_tokens = (0..9)
            .map(|_| {
                commands
                    .spawn((Name::new(format!("City {n}")), CityToken::new(player)))
                    .id()
            })
            .collect::<Vec<Entity>>();

        let (ship_stock, player_ships) = create_ship_stock(&mut commands, player);
        commands.entity(player).insert((
            TokenStock::new(token_count, tokens),
            CityTokenStock::new(9, city_tokens),
            ship_stock,
            player_ships,
        ));
    }
    info!("Done adding players");
}

pub fn setup_human_player(
    debug_options: &Res<DebugOptions>,
    trade_card_resource: &mut ResMut<CivilizationTradeCards>,
    commands: &mut Commands,
    player: Entity,
) {
    info!("Adding human player");
    commands.entity(player).remove::<StupidAi>();
    commands.entity(player).insert(IsHuman);
    if let Some(trade_cards) = &debug_options.human_trade_cards {
        let mut player_trade_cards = PlayerTradeCards::default();
        for (card, count) in trade_cards {
            for _ in 0..*count {
                player_trade_cards.add_trade_card(*card);
            }
        }
        commands.entity(player).insert(player_trade_cards);
    }

    if let Some(start_cards) = &debug_options.human_civ_cards {
        let mut player_civ_cards = PlayerCivilizationCards::default();
        for card in start_cards {
            player_civ_cards.cards.insert(*card);
        }
        commands.entity(player).insert(player_civ_cards);
    }

    if debug_options.human_starts_with_trade_cards {
        let mut player_trade_cards = PlayerTradeCards::default();
        (1..=9).for_each(|pile| {
            if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                player_trade_cards.add_trade_card(pulled_card);
            }
        });
        commands.entity(player).insert(player_trade_cards);
        commands.entity(player).insert(player_input_bundle());
    }
    info!("Added human player");
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct CardinalInput;

#[derive(InputAction)]
#[action_output(Vec3)]
struct MapInput;

#[derive(InputAction)]
#[action_output(bool)]
struct OkAction;

#[derive(InputAction)]
#[action_output(bool)]
struct CancelAction;

pub fn player_input_bundle() -> impl Bundle {
    (
        MovementInput,
        actions!(MovementInput[
            (
                Action::<CardinalInput>::new(),
                Bindings::spawn(Cardinal::wasd_keys()),
            ),(
                Action::<MapInput>::new(),
                Bindings::spawn(Spatial::new(KeyCode::KeyX, KeyCode::KeyZ, KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::ArrowUp, KeyCode::ArrowDown)),
            ),
            (
                Action::<OkAction>::new(),
                bindings![KeyCode::Enter],
            ),
                        (
                Action::<CancelAction>::new(),
                bindings![KeyCode::Escape],
            ),
        ]),
    )
}

pub fn connect_areas(
    mut area_query: Query<(Entity, &mut LandPassage, &mut SeaPassage, &NeedsConnections)>,
    named_areas: Query<(Entity, &GameArea)>,
    mut commands: Commands,
) {
    for (area_entity, mut land_passages, mut sea_passages, needed_connections) in &mut area_query {
        for named_area in &needed_connections.land_connections {
            for (target_area_entity, target_area) in named_areas.iter() {
                if target_area.id == *named_area {
                    land_passages.to_areas.push(target_area_entity);
                }
            }
        }
        for named_area in &needed_connections.sea_connections {
            for (target_area_entity, target_area) in named_areas.iter() {
                if target_area.id == *named_area {
                    sea_passages.add_passage(target_area_entity);
                }
            }
        }
        commands.entity(area_entity).remove::<NeedsConnections>();
    }
}

/// Lays each player's tokens out in their own column (`player_index * 15` on
/// x) with a vertical stack within it (`token_index * -5` on y), so different
/// civilizations' tokens visibly form independent piles instead of
/// overlapping (roadmap: "tokens of different civilizations should not stack
/// together").
///
/// `Population::player_tokens()` is a `HashMap<Entity, HashSet<Entity>>`, and
/// Rust's default hasher seeds every `HashMap`/`HashSet` instance randomly —
/// so iterating it directly assigns each player (and each token within a
/// player) an *unstable* index that can differ across recomputations even
/// though the underlying tokens haven't moved. That's what the "this
/// information is thrown away in some circumstances" complaint was: it
/// wasn't lost, it was re-shuffled every time this system re-ran. Sorting by
/// `Entity` before indexing makes the layout deterministic and stable.
pub fn fix_token_positions(
    population_query: Query<(Entity, &Population, &Transform, &FixTokenPositions), Without<Token>>,
    mut token_transform_query: Query<&mut Transform, With<Token>>,
    mut commands: Commands,
) {
    for (area_entity, pop, area_transform, _) in population_query.iter() {
        let mut players: Vec<(&Entity, &HashSet<Entity>)> = pop.player_tokens().iter().collect();
        players.sort_by_key(|(player, _)| **player);
        for (player_index, (_, tokens)) in players.into_iter().enumerate() {
            let mut sorted_tokens: Vec<&Entity> = tokens.iter().collect();
            sorted_tokens.sort();
            for (token_index, token) in sorted_tokens.into_iter().enumerate() {
                if let Ok(mut token_transform) = token_transform_query.get_mut(*token) {
                    token_transform.translation = area_transform.translation
                        + vec3(
                            (player_index * 15) as f32,
                            ((token_index as i32) * -5) as f32,
                            0.0,
                        );
                }
            }
        }
        commands.entity(area_entity).remove::<FixTokenPositions>();
    }
}

/**
This is 100% needed to be able to test expansion and stuff.
*/
pub fn move_tokens_from_stock_to_area(
    mut move_commands: MessageReader<MoveTokensFromStockToAreaCommand>,
    mut population_query: Query<(&mut Population, &Transform)>,
    mut player_query: Query<(&mut PlayerAreas, &mut TokenStock, &Faction)>,
    mut commands: Commands,
    game_factions: Res<AvailableFactions>,
) {
    for ev in move_commands.read() {
        if let Ok((mut player_areas, mut stock, faction)) = player_query.get_mut(ev.player_entity)
            && let Ok((mut population, area_transform)) = population_query.get_mut(ev.area_entity)
            && let Some(tokens_to_move) = stock.remove_tokens_from_stock(ev.number_of_tokens)
        {
            tokens_to_move.iter().for_each(|t| {
                population.add_token_to_area(ev.player_entity, *t);
                player_areas.add_token_to_area(ev.area_entity, *t);
                commands.entity(*t).insert((
                    Sprite {
                        image: game_factions
                            .faction_icons
                            .get(&faction.faction)
                            .unwrap()
                            .clone(),
                        ..default()
                    },
                    Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                        .with_translation(area_transform.translation),
                ));
            });
        }
        commands.entity(ev.area_entity).insert(FixTokenPositions);
    }
}

pub fn print_names_of_phases(
    mut state_transition_event: MessageReader<StateTransitionEvent<GameActivity>>,
    debug_options: Res<DebugOptions>,
) {
    if debug_options.log_selected_moves {
        for event in state_transition_event.read() {
            info!("Went from: {:#?} to {:#?}", event.exited, event.entered);
        }
    }
}

#[cfg(test)]
mod fix_token_positions_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::World;

    /// Two players, several tokens each, sharing one area: different
    /// civilizations must not overlap (distinct x per player), and re-running
    /// the system must not reshuffle anyone (regression test for the
    /// HashMap-iteration-order bug described above `fix_token_positions`).
    #[test]
    fn different_players_get_distinct_stable_columns() {
        let mut world = World::new();
        let area_transform = Transform::from_xyz(100.0, 200.0, 0.0);

        let player_a = world.spawn_empty().id();
        let player_b = world.spawn_empty().id();

        let mut population = Population::new(20);
        let a_tokens: Vec<Entity> = (0..3)
            .map(|_| {
                world
                    .spawn((Token::new(player_a), Transform::default()))
                    .id()
            })
            .collect();
        let b_tokens: Vec<Entity> = (0..2)
            .map(|_| {
                world
                    .spawn((Token::new(player_b), Transform::default()))
                    .id()
            })
            .collect();
        for &t in &a_tokens {
            population.add_token_to_area(player_a, t);
        }
        for &t in &b_tokens {
            population.add_token_to_area(player_b, t);
        }

        let area = world
            .spawn((population, area_transform, FixTokenPositions))
            .id();

        world.run_system_once(fix_token_positions).unwrap();

        // Exact-integer offsets (index * a small constant): comparing as i64
        // sidesteps clippy's float_cmp lint without weakening the assertion.
        let x_of =
            |world: &World, token: Entity| world.get::<Transform>(token).unwrap().translation.x as i64;

        // Every token of a given player shares that player's column (x).
        let a_x = x_of(&world, a_tokens[0]);
        for &t in &a_tokens {
            assert_eq!(x_of(&world, t), a_x, "player A tokens must share a column");
        }
        let b_x = x_of(&world, b_tokens[0]);
        for &t in &b_tokens {
            assert_eq!(x_of(&world, t), b_x, "player B tokens must share a column");
        }
        // The two players must not share a column.
        assert_ne!(a_x, b_x, "different players must not overlap");

        // The marker is consumed.
        assert!(world.get::<FixTokenPositions>(area).is_none());

        // Re-running after re-adding the marker (as a later phase would)
        // must reproduce the exact same layout, not reshuffle it.
        world.entity_mut(area).insert(FixTokenPositions);
        world.run_system_once(fix_token_positions).unwrap();
        assert_eq!(x_of(&world, a_tokens[0]), a_x, "layout must be stable across re-runs");
        assert_eq!(x_of(&world, b_tokens[0]), b_x, "layout must be stable across re-runs");
    }

    /// Within one player's stack, tokens get distinct y offsets — no two
    /// tokens land on the exact same spot.
    #[test]
    fn tokens_within_a_player_stack_vertically_without_overlap() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let mut population = Population::new(20);
        let tokens: Vec<Entity> = (0..4)
            .map(|_| {
                world
                    .spawn((Token::new(player), Transform::default()))
                    .id()
            })
            .collect();
        for &t in &tokens {
            population.add_token_to_area(player, t);
        }
        world.spawn((population, Transform::default(), FixTokenPositions));

        world.run_system_once(fix_token_positions).unwrap();

        let ys: HashSet<i64> = tokens
            .iter()
            .map(|&t| world.get::<Transform>(t).unwrap().translation.y as i64)
            .collect();
        assert_eq!(ys.len(), tokens.len(), "no two tokens in a stack overlap");
    }
}
