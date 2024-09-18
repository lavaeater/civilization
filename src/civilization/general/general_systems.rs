use crate::civilization::census::census_components::Census;
use crate::civilization::general::general_components::{CitySite, CityToken, CityTokenStock, Faction, GameArea, LandPassage, NeedsConnections, PlayerAreas, PlayerCities, Population, StartArea, Token, Treasury};
use crate::civilization::general::general_components::PlayerStock;
use bevy::core::Name;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, StateTransitionEvent, With};
use bevy::utils::HashMap;
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::general::general_enums::GameFaction::{Crete, Egypt};
use crate::civilization::general::general_events::{MoveTokensFromStockToAreaCommand, ReturnTokenToStock};
use crate::GameActivity;
use crate::player::Player;

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
    mut commands: Commands
) {
    (1..=2).for_each(|n| {
        // Create Player
        let player = commands
            .spawn(
                (
                    Player {},
                    Name::new(format!("p{n}")),
                    Census { population: 0 },
                    Treasury { tokens: vec![] },
                    Faction { faction: if n % 2 == 0 { Egypt } else { Crete } },
                    PlayerAreas::default(),
                    PlayerCities::default()
                )
            ).id();

        let tokens = (0..3).map(|_| {
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

pub fn setup_game(
    mut commands: Commands,
) {
    /*
    Areas | Connected To
    ---------------------
    egypt | numidia, cyprus, syria
    crete | cyprus, thrace, athens
    numidia | egypt, iberia
    cyprus | egypt, crete, syria
    syria | egypt, cyprus, thrace
    thrace | syria, crete, athens
    athens | thrace, crete
    iberia | numidia
     */
    let map: HashMap<String, Vec<String>> = [
        ("egypt", vec!["alexandria"]),
        ("crete", vec!["cyprus", "thrace", "athens"]),
        ("numidia", vec!["alexandria", "iberia"]),
        ("cyprus", vec!["egypt", "crete", "syria"]),
        ("syria", vec!["egypt", "cyprus", "thrace"]),
        ("thrace", vec!["syria", "crete", "athens"]),
        ("athens", vec!["thrace", "crete"]),
        ("iberia", vec!["numidia"]),
        ("alexandria", vec!["egypt", "numidia", "cyprus", "syria"]),
    ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.into_iter().map(|s| s.to_string()).collect()))
        .collect();

    let city_sites = ["crete", "athens", "alexandria", "iberia"];

    for (area, connections) in map {
        match area.as_str() {
            "egypt" => {
                commands
                    .spawn(
                        (
                            Name::new("egypt"),
                            GameArea {},
                            LandPassage::default(),
                            NeedsConnections {
                                land_connections: connections,
                                sea_connections: vec!(),
                            },
                            StartArea {
                                faction: Egypt
                            },
                            Population::new(4)
                        )
                    );
            }
            "crete" => {
                commands
                    .spawn(
                        (
                            Name::new("crete"),
                            GameArea {},
                            LandPassage::default(),
                            NeedsConnections {
                                land_connections: connections,
                                sea_connections: vec!(),
                            },
                            StartArea {
                                faction: Crete
                            },
                            Population::new(3)
                        )
                    );
            }
            _ => {
                let area_id = commands
                    .spawn(
                        (
                            GameArea {},
                            Name::new(area.clone()),
                            LandPassage::default(),
                            NeedsConnections {
                                land_connections: connections,
                                sea_connections: vec!(),
                            },
                            Population::new(3)
                        )
                    ).id();
                if city_sites.contains(&&*area) {
                    commands.entity(area_id).insert(CitySite {});
                }
            }
        }
    }
}

pub fn connect_areas(
    mut area_query: Query<(Entity, &mut LandPassage, &NeedsConnections)>,
    named_areas: Query<(Entity, &Name), With<GameArea>>,
    mut commands: Commands,
) {
    for (area_entity,
        mut land_passages,
        needed_connections) in area_query.iter_mut() {
        for named_area in needed_connections.land_connections.clone().into_iter() {
            let na = Name::new(named_area.clone());
            //This is fucking stupid, but who cares?
            for (target_area_entity, target_name) in named_areas.iter() {
                if *target_name == na {
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
            if let Ok(mut stock) = stock_query.get_mut(token.player) {
                stock.tokens.push(return_event.token_entity);
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