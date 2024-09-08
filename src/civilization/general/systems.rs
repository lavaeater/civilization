use crate::civilization::census::components::Census;
use crate::civilization::general::components::{GameArea, CitySite, CityToken, CityTokenStock, Faction, LandPassage, NeedsConnections, Population, StartArea, Token, Treasury, PlayerAreas, PlayerCities};
use crate::civilization::general::components::Stock;
use bevy::core::Name;
use bevy::prelude::{Commands, Entity, EventReader, Query, With};
use bevy::utils::HashMap;
use crate::civilization::general::enums::GameFaction::{Crete, Egypt};
use crate::civilization::general::events::{MoveTokensFromStockToAreaCommand, ReturnTokenToStock};
use crate::player::Player;

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
                    Stock::new(
                    47,
                    tokens),
                    CityTokenStock::new(
                        9,
                        city_tokens
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
    mut stock_query: Query<&mut Stock>,
    mut population_query: Query<&mut Population>,
    mut presence_query: Query<&mut PlayerAreas>
) {
    for ev in move_commands.read() {
        if let Ok(mut stock) = stock_query.get_mut(ev.player_entity) {
            if let Ok(mut population) = population_query.get_mut(ev.area_entity) {
                if stock.tokens.len() > ev.number_of_tokens {
                    let tokens_to_move = (0..ev.number_of_tokens).map(|_| stock.tokens.swap_remove(0)).collect::<Vec<Entity>>();
                    if !population.player_tokens.contains_key(&ev.player_entity) {
                        population.player_tokens.insert(ev.player_entity, Vec::new());
                    }
                    let mut player_areas = presence_query.get_mut(ev.player_entity).unwrap();
                    player_areas.areas.insert(ev.area_entity);
                    tokens_to_move
                        .iter()
                        .for_each(|t| {
                            population
                                .player_tokens
                                .get_mut(&ev.player_entity)
                                .unwrap()
                                .push(*t)
                        });
                }
            }
        }
    }
}

pub(crate) fn return_token_to_stock(
    mut event: EventReader<ReturnTokenToStock>,
    mut stock_query: Query<&mut Stock>,
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
