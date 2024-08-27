use crate::civilization::census::components::Census;
use crate::civilization::general::components::{Area, LandPassage, NeedsConnections, Population, StartArea, Token};
use crate::civilization::general::components::Stock;
use bevy::core::Name;
use bevy::prelude::{Commands, Entity, EventReader, Query, With};
use crate::civilization::general::events::MoveTokensFromStockToAreaCommand;

pub fn setup_players(
    mut commands: Commands
) {
    (1..=1).for_each(|n| {
        // Create Player
        let player = commands
            .spawn(
                (
                    crate::player::Player {},
                    Name::new(format!("Player {n}")),
                    Census { population: 0 },
                )
            ).id();

        let tokens = (0..47).map(|_| {
            commands
                .spawn(
                    (
                        Name::new(format!("Token {n}")),
                        Token { player })).id()
        }
        )
            .collect::<Vec<Entity>>();
        commands
            .entity(player)
            .insert(
                Stock::new(
                    47,
                    tokens,
                )
            );
    });
}

pub fn setup_game(
    mut commands: Commands,
) {
    // Create some Areas
    commands
        .spawn(
            (
                Name::new("sa"),
                Area { max_population: 2 },
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("2".into(), "3".into()),
                    sea_connections: vec!(),
                },
                StartArea {},
                Population::default()
            )
        );
    commands
        .spawn(
            (
                Area { max_population: 3 },
                Name::new("2"),
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("sa".into(), "4".into()),
                    sea_connections: vec!(),
                },
                Population::default()
            )
        );
    commands
        .spawn(
            (
                Area { max_population: 1 },
                Name::new("3"),
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("sa".into(), "4".into()),
                    sea_connections: vec!(),
                },
                Population::default()
            )
        );
    commands
        .spawn(
            (
                Area { max_population: 5 },
                Name::new("4"),
                LandPassage::default(),
                NeedsConnections {
                    land_connections: vec!("2".into(), "3".into()),
                    sea_connections: vec!(),
                },
                Population::default()
            )
        );
}

pub fn connect_areas(
    mut area_query: Query<(Entity, &mut LandPassage, &NeedsConnections)>,
    named_areas: Query<(Entity, &Name), With<Area>>,
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
) {
    for ev in move_commands.read() {
        if let Ok(mut stock) = stock_query.get_mut(ev.player_entity) {
            if let Ok(mut population) = population_query.get_mut(ev.area_entity) {
                let tokens_to_move = (0..ev.number_of_tokens).map(|_| stock.tokens.swap_remove(0)).collect::<Vec<Entity>>();
                if !population.player_tokens.contains_key(&ev.player_entity) {
                   population.player_tokens.insert(ev.player_entity, Vec::new());
                }
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
