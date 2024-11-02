use crate::civilization::components::prelude::*;
use crate::civilization::concepts::census::components::Census;
use crate::civilization::concepts::map::map_plugin::AvailableFactions;
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::events::prelude::*;
use crate::player::Player;
use crate::stupid_ai::prelude::*;
use crate::GameActivity;
use bevy::core::Name;
use bevy::math::{vec3, Vec3};
use bevy::prelude::{debug, default, info, Commands, Entity, EventReader, EventWriter, NextState, Query, Res, ResMut, StateTransitionEvent, Transform, With, Without};
use bevy::sprite::SpriteBundle;
use rand::seq::IteratorRandom;
use crate::civilization::enums::prelude::GameFaction;

pub fn start_game(
    player_query: Query<(Entity, &Name, &Faction), With<Player>>,
    start_area_query: Query<(Entity, &Name, &StartArea)>,
    mut writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>) {
    debug!("4. Starting the game!");
    for (player_entity, name, player_faction) in player_query.iter() {
        debug!("Starting the game for player: {:?}", name);
        if let Some((area_entity, area_name, _)) = start_area_query.iter().find(|(_, _, start_area)| start_area.faction == player_faction.faction) {
            debug!("Putting a token in: {:?}", area_name);
            writer.send(
                MoveTokensFromStockToAreaCommand {
                    area_entity,
                    player_entity,
                    number_of_tokens: 1,
                });
        }
    }
    next_state.set(GameActivity::PopulationExpansion);
}

pub fn setup_players(
    mut commands: Commands,
    mut available_factions: ResMut<AvailableFactions>,
) {
    debug!("3. Setting up players!");
    (1..=8).for_each(|n| {
        if let Some(faction) = available_factions.remaining_factions.clone().iter().choose(&mut rand::thread_rng()) {
            available_factions.remaining_factions.remove(faction);
            // Create Player
            let player = commands
                .spawn(
                    (
                        Player,
                        Name::new(format!("p_{:?}_{n}", faction)),
                        Census { population: 0 },
                        Treasury::default(),
                        Faction::new(*faction),
                        PlayerAreas::default(),
                        PlayerCities::default(),
                        StupidAi,
                        PlayerTradeCards::default(),
                    )
                ).id();

            if *faction == GameFaction::Egypt {
                commands.entity(player).insert(IsHuman);
            }

            let tokens = (0..47).map(|_| {
                commands
                    .spawn(
                        (
                            Name::new(format!("Token {n}")),
                            Token::new(player))).id()
            }
            )
                .collect::<Vec<Entity>>();

            let city_tokens = (0..9).map(|_| {
                commands
                    .spawn(
                        (
                            Name::new(format!("City {n}")),
                            CityToken::new(player))).id()
            }
            )
                .collect::<Vec<Entity>>();
            commands
                .entity(player)
                .insert(
                    (
                        TokenStock::new(
                            47,
                            tokens),
                        CityTokenStock::new(
                            9,
                            city_tokens,
                        )
                    )
                );
        }
    });
    debug!("Players are set up!");
}

pub fn connect_areas(
    mut area_query: Query<(Entity, &mut LandPassage, &NeedsConnections)>,
    named_areas: Query<(Entity, &GameArea)>,
    mut commands: Commands,
) {
    for (area_entity,
        mut land_passages,
        needed_connections) in area_query.iter_mut() {
        for named_area in needed_connections.land_connections.iter() {
            //This is fucking stupid, but who cares?
            for (target_area_entity, target_area) in named_areas.iter() {
                if target_area.id == *named_area {
                    land_passages.to_areas.push(target_area_entity);
                }
            }
        }
        commands.entity(area_entity).remove::<NeedsConnections>();
    }
}

pub fn fix_token_positions(
    population_query: Query<(Entity, &Population, &Transform, &FixTokenPositions), Without<Token>>,
    mut token_transform_query: Query<&mut Transform, With<Token>>,
    mut commands: Commands,
) {
    for (area_entity, pop, area_transform, _) in population_query.iter() {
        for (player_index, (_, tokens)) in pop.player_tokens().iter().enumerate() {
            for (token_index, token) in tokens.iter().enumerate() {
                if let Ok(mut token_transform) = token_transform_query.get_mut(*token) {
                    token_transform.translation = area_transform.translation + vec3(
                        (player_index * 15) as f32,
                        ((token_index as i32) * -5) as f32,
                        0.0);
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
    mut move_commands: EventReader<MoveTokensFromStockToAreaCommand>,
    mut population_query: Query<(&mut Population, &Transform)>,
    mut player_query: Query<(&mut PlayerAreas, &mut TokenStock, &Faction)>,
    mut commands: Commands,
    game_factions: Res<AvailableFactions>,
) {
    for ev in move_commands.read() {
        if let Ok((mut player_areas, mut stock, faction)) = player_query.get_mut(ev.player_entity) {
            if let Ok((mut population, area_transform)) = population_query.get_mut(ev.area_entity) {
                if let Some(tokens_to_move) = stock.remove_tokens_from_stock(ev.number_of_tokens) {
                    tokens_to_move
                        .iter()
                        .for_each(|t| {
                            population.add_token_to_area(ev.player_entity, *t);
                            player_areas.add_token_to_area(ev.area_entity, *t);
                            commands.entity(*t).insert(SpriteBundle {
                                texture: game_factions.faction_icons.get(&faction.faction).unwrap().clone(),
                                transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25))
                                    .with_translation(area_transform.translation),
                                ..default()
                            });
                        });
                }
            }
        }
        commands.entity(ev.area_entity).insert(FixTokenPositions);
    }
}

pub fn print_names_of_phases(
    mut state_transition_event: EventReader<StateTransitionEvent<GameActivity>>,
) {
    for event in state_transition_event.read() {
        info!("Went from: {:?} to {:?}", event.exited, event.entered);
    }
}