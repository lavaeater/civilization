use crate::civilization::census::census_components::Census;
use crate::civilization::general::general_components::{CityToken, CityTokenStock, Faction, GameArea, LandPassage, NeedsConnections, PlayerAreas, PlayerCities, Population, StartArea, Token, Treasury};
use crate::civilization::general::general_components::PlayerStock;
use bevy::core::Name;
use bevy::prelude::{debug, Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, StateTransitionEvent, With};
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::general::general_events::{MoveTokensFromStockToAreaCommand, ReturnTokenToStock};
use crate::civilization::map::map_plugin::AvailableFactions;
use crate::GameActivity;
use crate::player::Player;
use rand::seq::IteratorRandom;
use crate::stupid_ai::stupid_ai_plugin::StupidAi;

pub fn start_game(
    player_query: Query<(Entity, &Name, &Faction), With<Player>>,
    start_area_query: Query<(Entity, &Name, &StartArea)>,
    mut writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>) {
    for (player_entity, _name, player_faction) in player_query.iter() {
        if let Some((area_entity, _area_name, _)) = start_area_query.iter().find(|(_, _, start_area)| start_area.faction == player_faction.faction) {
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
    mut available_factions: ResMut<AvailableFactions>
) {
    (1..=2).for_each(|n| {
        
        let faction = *available_factions.remaining_factions.iter().choose(&mut rand::thread_rng()).unwrap();
        available_factions.remaining_factions.remove(&faction);
        // Create Player
        let player = commands
            .spawn(
                (
                    Player,
                    Name::new(format!("p_{n}")),
                    Census { population: 0 },
                    Treasury::default(),
                    Faction::new(faction),
                    PlayerAreas::default(),
                    PlayerCities::default(),
                    StupidAi
                )
            ).id();

        // if n % 2 == 0 {
        //     commands.entity(player).insert(StupidAi::default());
        // }

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
                    PlayerStock::new(
                        47,
                        tokens),
                    CityTokenStock::new(
                        9,
                        city_tokens,
                    )
                )
            );
    });
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

/**
This is 100% needed to be able to test expansion and stuff.
*/
pub fn move_tokens_from_stock_to_area(
    mut move_commands: EventReader<MoveTokensFromStockToAreaCommand>,
    mut stock_query: Query<&mut PlayerStock>,
    mut population_query: Query<&mut Population>,
    mut player_areas_query: Query<&mut PlayerAreas>,
) {
    for ev in move_commands.read() {
        if let Ok(mut stock) = stock_query.get_mut(ev.player_entity) {
            if let Ok(mut population) = population_query.get_mut(ev.area_entity) {
                if let Some(tokens_to_move) = stock.remove_tokens_from_stock(ev.number_of_tokens) {
                    let mut player_areas = player_areas_query.get_mut(ev.player_entity).unwrap();
                    tokens_to_move
                        .iter()
                        .for_each(|t| {
                            population.add_token_to_area(ev.player_entity, *t);
                            player_areas.add_token_to_area(ev.area_entity, *t);
                        });
                }
            }
        }
    }
}

pub(crate) fn return_token_to_stock(
    mut event: EventReader<ReturnTokenToStock>,
    mut stock_query: Query<&mut PlayerStock>,
    token_query: Query<&Token>,
) {
    for return_event in event.read() {
        if let Ok(token) = token_query.get(return_event.token_entity) {
            debug!("we have token component!");
            if let Ok(mut stock) = stock_query.get_mut(token.player) {
                debug!("we return it");
                stock.return_token_to_stock(return_event.token_entity);
            }
        }
    }
}

pub fn print_names_of_phases(
    mut write_line: EventWriter<PrintConsoleLine>,
    mut state_transition_event: EventReader<StateTransitionEvent<GameActivity>>,
) {
    for event in state_transition_event.read() {
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Went from: {:?} to {:?}", event.exited, event.entered))));
    }
}