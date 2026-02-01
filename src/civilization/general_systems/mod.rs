use crate::civilization::components::population::Population;
use crate::civilization::components::*;
use crate::civilization::concepts::acquire_trade_cards::trade_card_components::{
    CivilizationTradeCards, PlayerTradeCards,
};
use crate::civilization::concepts::census::census_components::Census;
use crate::civilization::concepts::map::map_plugin::AvailableFactions;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::plugins::civilization_plugin::DebugOptions;
use crate::player::Player;
use crate::stupid_ai::prelude::*;
use crate::GameActivity;
use bevy::math::{vec3, Vec3};
use bevy::prelude::{debug, default, info, Commands, Entity, MessageReader, MessageWriter, Name, NextState, Query, Res, ResMut, Sprite, StateTransitionEvent, Transform, With, Without};
use rand::seq::IteratorRandom;

pub fn start_game(
    player_query: Query<(Entity, &Name, &Faction), With<Player>>,
    start_area_query: Query<(Entity, &Name, &StartArea)>,
    mut writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    debug!("4. Starting the game!");
    for (player_entity, name, player_faction) in player_query.iter() {
        debug!("Starting the game for player: {:#?}", name);
        if let Some((area_entity, area_name, _)) = start_area_query
            .iter()
            .find(|(_, _, start_area)| start_area.faction == player_faction.faction)
        {
            debug!("Putting a token in: {:#?}", area_name);
            writer.write(MoveTokensFromStockToAreaCommand {
                area_entity,
                player_entity,
                number_of_tokens: 1,
            });
        }
    }
    next_state.set(GameActivity::PopulationExpansion);
}

pub fn start_game_after_player_setup(
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    next_state.set(GameActivity::StartGame);
}

pub fn setup_players(
    debug_options: Res<DebugOptions>,
    mut trade_card_resource: ResMut<CivilizationTradeCards>,
    mut commands: Commands,
    mut available_factions: ResMut<AvailableFactions>,
) {
    debug!("3. Setting up players!");
    (1..=debug_options.number_of_players).for_each(|n| {
        if let Some(faction) = available_factions
            .remaining_factions
            .clone()
            .iter()
            .choose(&mut rand::rng())
        {
            available_factions.remaining_factions.remove(faction);
            // Create Player
            let player = commands
                .spawn((
                    Player,
                    Name::new(format!("p_{:#?}_{n}", faction)),
                    Census { population: 0 },
                    Treasury::default(),
                    Faction::new(*faction),
                    PlayerAreas::default(),
                    PlayerCities::default(),
                    StupidAi,
                    PlayerTradeCards::default(),
                ))
                .id();

            if debug_options.add_human_player && *faction == debug_options.human_faction {
                commands.entity(player).insert(IsHuman);
                if debug_options.human_starts_with_trade_cards {
                    let mut player_trade_cards = PlayerTradeCards::default();
                    (1..=9).for_each(|pile| {
                        if let Some(pulled_card) = trade_card_resource.pull_card_from(pile) {
                            player_trade_cards.add_trade_card(pulled_card);
                        }
                    });
                    commands.entity(player).insert(player_trade_cards);
                }
            }

            let tokens = (0..47)
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
            commands.entity(player).insert((
                TokenStock::new(47, tokens),
                CityTokenStock::new(9, city_tokens),
            ));
        }
    });
}

pub fn connect_areas(
    mut area_query: Query<(Entity, &mut LandPassage, &NeedsConnections)>,
    named_areas: Query<(Entity, &GameArea)>,
    mut commands: Commands,
) {
    for (area_entity, mut land_passages, needed_connections) in area_query.iter_mut() {
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
        if let Ok((mut player_areas, mut stock, faction)) = player_query.get_mut(ev.player_entity) {
            if let Ok((mut population, area_transform)) = population_query.get_mut(ev.area_entity) {
                if let Some(tokens_to_move) = stock.remove_tokens_from_stock(ev.number_of_tokens) {
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
            }
        }
        commands.entity(ev.area_entity).insert(FixTokenPositions);
    }
}

pub fn print_names_of_phases(
    mut state_transition_event: MessageReader<StateTransitionEvent<GameActivity>>,
) {
    for event in state_transition_event.read() {
        info!("Went from: {:#?} to {:#?}", event.exited, event.entered);
    }
}
