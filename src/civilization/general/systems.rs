use bevy::core::Name;
use bevy::hierarchy::Children;
use bevy::prelude::{BuildChildren, Commands, Entity, EventReader, Query, With};
use crate::civilization::census::components::Census;
use crate::civilization::general::plugin::{Population, Area, StartArea, LandPassage, MoveTokensFromStockToAreaCommand, NeedsConnections, Stock, Token};

pub fn setup_players(
    mut commands: Commands
) {
    (1..=1).into_iter().for_each(|n| {
        // Create Player
        let player = commands
            .spawn(
                (
                    crate::player::Player {},
                    Name::new(format!("Player {n}")),
                    Census { population: 0 },
                )
            ).id();

        let tokens =                     (0..47).map(|_| {
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
                    tokens
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
    player_stock_query: Query<&Children, With<Stock>>,
    player_query: Query<&Children>,
    mut commands: Commands,
) {
    for ev in move_commands.read() {
        if let Ok(children) = player_query.get(ev.stock_entity) {
            for child in children {
                if let Ok(tokens) = player_stock_query.get(*child) {
                    let tokens_to_move = &tokens.into_iter().as_slice()[0..ev.number_of_tokens];
                    commands.entity(ev.stock_entity).remove_children(tokens_to_move);
                    commands.entity(ev.population_entity).push_children(tokens_to_move);
                }
            }
        }
    }
}