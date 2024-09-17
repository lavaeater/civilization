extern crate rspec;

mod common;

use bevy::prelude::{Events, Update};
use bevy_game::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use bevy_game::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use bevy_game::civilization::game_moves::game_moves_systems::recalculate_pop_exp_moves_for_player;
use bevy_game::civilization::general::general_components::{PlayerAreas, Population, PlayerStock};
use bevy_game::civilization::general::general_enums::GameFaction;
use crate::common::{create_area, setup_bevy_app, setup_player};

#[test]        
fn main() {
    #[derive(Clone, Default, Debug)]
    struct Environment;

    rspec::run(&rspec::given("game_moves_plugin", Environment::default(), |ctx| {
        ctx.when("a player with token in three areas", |ctx| {
            ctx.when("he does not have enought tokens for expansion", |ctx| {
                ctx.then("the correct moves are created", |_| {
                    let mut app = setup_bevy_app(|mut app| {
                        app
                            .add_event::<RecalculatePlayerMoves>()
                            .add_systems(Update, recalculate_pop_exp_moves_for_player)
                        ;
                        app
                    });

                    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

                    let mut player_areas = PlayerAreas::default();
                    let mut stock = PlayerStock::new(47, tokens.drain(0..4).collect());

                    let area_one = create_area(&mut app, "Egypt");
                    let area_two = create_area(&mut app, "Thrace");
                    let area_three = create_area(&mut app, "Crete");
                    let areas = vec![area_one, area_two, area_three];
                    for area in areas.iter() {
                        let mut population = Population::new(4);
                        let token = stock.remove_token_from_stock().unwrap();
                        player_areas.add_token_to_area(*area, token);
                        population.add_token_to_area(player, token);
                        app
                            .world_mut()
                            .entity_mut(*area)
                            .insert(population);
                    }

                    app.world_mut()
                        .entity_mut(player)
                        .insert((player_areas, stock));

                    
                    let mut events = app.world_mut()
                        .resource_mut::<Events<RecalculatePlayerMoves>>();

                    events.send(RecalculatePlayerMoves::new(player));

                    // Act
                    app.update();
                    // Assert
                    let player_moves = app
                        .world()
                        .entity(player)
                        .get::<AvailableMoves>();
                    assert!(player_moves.is_some());
                    let player_moves = player_moves.unwrap();
                    assert_eq!(player_moves.moves.len(), 3);
                    for (move_index, p_move) in player_moves.moves.iter() {
                        assert_eq!(*move_index, 1);
                        assert!(matches!(p_move, Move::PopulationExpansion(..)));
                        match p_move {
                            Move::PopulationExpansion(_move_area, tokens) => {
                                assert_eq!(*tokens, 1);
                            }
                        };
                    }
                });
            });
        });
    }));
}


#[test]
fn given_a_player_with_too_few_tokens_for_expansion_the_correct_moves_are_created() {
    let mut app = setup_bevy_app(|mut app| {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(Update, recalculate_pop_exp_moves_for_player)
        ;
        app
    });

    let (player, mut tokens, _city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);

    let mut player_areas = PlayerAreas::default();
    let mut stock = PlayerStock::new(47, tokens.drain(0..4).collect());

    let area = create_area(&mut app, "Egypt");
    let mut population = Population::new(4);

    let tokens_to_add = stock.remove_tokens_from_stock(2).unwrap();
    tokens_to_add.iter().for_each(|token| {
        player_areas.add_token_to_area(area, *token);
        population.add_token_to_area(player, *token);
    });

    app.world_mut()
        .entity_mut(player)
        .insert((player_areas, stock));

    app
        .world_mut()
        .entity_mut(area)
        .insert(population);

    let mut events = app.world_mut()
        .resource_mut::<Events<RecalculatePlayerMoves>>();

    events.send(RecalculatePlayerMoves::new(player));

    // Act
    app.update();
    // Assert
    let player_moves = app
        .world()
        .entity(player)
        .get::<AvailableMoves>();
    assert!(player_moves.is_some());
    let player_moves = player_moves.unwrap();
    assert_eq!(player_moves.moves.len(), 1);
    let first_move = player_moves.moves.values().collect().first().unwrap();
    assert!(matches!(*first_move, Move::PopulationExpansion(..)));
    match *first_move {
        Move::PopulationExpansion( move_area, tokens) => {
            assert_eq!(tokens, 2);
            assert_eq!(move_area, area);
        }
    };
}