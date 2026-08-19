// Rule 35 — Victory Determination. `determine_winner` sums, per player:
//   A. civilization card face value
//   B. commodity card set value (28.51)
//   C. treasury tokens (1 pt each)
//   D. A.S.T. position (100 pts/space)
//   E. cities on board (50 pts each)
// then ranks by total, tiebreaking on furthest A.S.T. position (35.2).

use crate::{setup_bevy_app, setup_player};
use adv_civ::civilization::{
    determine_winner, AstPosition, AvailableCivCards, CivCardDefinition, CivCardName, CivCardType,
    GameFaction, GameResult, PlayerCivilizationCards, PlayerTradeCards, TradeCard,
};
use bevy::app::Update;
use bevy::platform::collections::HashSet;
use bevy::prelude::Entity;

fn setup_app() -> bevy::prelude::App {
    setup_bevy_app(|mut app| {
        app.add_systems(Update, determine_winner);
        app
    })
}

fn add_treasury_tokens(app: &mut bevy::prelude::App, player: Entity, count: usize) {
    let tokens: Vec<Entity> = (0..count).map(|_| app.world_mut().spawn_empty().id()).collect();
    let world = app.world_mut();
    let mut entity_mut = world.entity_mut(player);
    let mut treasury = entity_mut.get_mut::<adv_civ::civilization::Treasury>().unwrap();
    for token in tokens {
        treasury.add_token_to_treasury(token);
    }
}

#[test]
fn scores_ast_position_and_cities_with_no_cards_or_treasury() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(player).insert(AstPosition::new(3));
    // setup_player already gives 9 spare city-token entities in stock, but
    // PlayerCities (cities actually *built* on the board) starts empty.

    app.update();

    let result = app.world().resource::<GameResult>();
    let (name, total, ast_space) = &result.standings[0];
    assert_eq!(name, "Egypt");
    assert_eq!(*ast_space, 3);
    // D only: 3 * 100 = 300. No civ cards, no commodities, no treasury, no cities.
    assert_eq!(*total, 300);
}

#[test]
fn scores_treasury_tokens_at_one_point_each() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(player).insert(AstPosition::new(0));
    add_treasury_tokens(&mut app, player, 12);

    app.update();

    let result = app.world().resource::<GameResult>();
    assert_eq!(result.standings[0].1, 12);
}

#[test]
fn scores_civilization_cards_at_face_value() {
    let mut app = setup_app();
    app.insert_resource(AvailableCivCards {
        cards: vec![
            CivCardDefinition {
                name: CivCardName::Pottery,
                description: String::new(),
                card_type: CivCardType::Crafts.into(),
                cost: 30,
                credits: vec![],
                prerequisites: vec![],
            },
            CivCardDefinition {
                name: CivCardName::Mining,
                description: String::new(),
                card_type: CivCardType::Crafts.into(),
                cost: 220,
                credits: vec![],
                prerequisites: vec![],
            },
        ],
    });
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(player).insert(AstPosition::new(0));
    let mut cards = HashSet::new();
    cards.insert(CivCardName::Pottery);
    cards.insert(CivCardName::Mining);
    app.world_mut()
        .entity_mut(player)
        .insert(PlayerCivilizationCards { cards });

    app.update();

    let result = app.world().resource::<GameResult>();
    // 30 + 220 = 250, no other components at play.
    assert_eq!(result.standings[0].1, 250);
}

#[test]
fn scores_commodity_card_sets_by_count_squared_times_face_value() {
    let mut app = setup_app();
    let (player, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(player).insert(AstPosition::new(0));
    let mut trade_cards = PlayerTradeCards::default();
    // 3 Grain (value 4 each): set value = 3^2 * 4 = 36 (rule 28.51).
    trade_cards.add_trade_cards(TradeCard::Grain, 3);
    app.world_mut().entity_mut(player).insert(trade_cards);

    app.update();

    let result = app.world().resource::<GameResult>();
    assert_eq!(result.standings[0].1, 36);
}

#[test]
fn winner_is_highest_total_not_highest_ast_position() {
    let mut app = setup_app();
    let (leader, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(leader).insert(AstPosition::new(1));
    add_treasury_tokens(&mut app, leader, 500); // 100 (AST) + 500 (treasury) = 600

    let (runner_up, _, _) = setup_player(&mut app, "Crete", GameFaction::Crete);
    app.world_mut().entity_mut(runner_up).insert(AstPosition::new(4)); // 400, further along but fewer points

    app.update();

    let result = app.world().resource::<GameResult>();
    assert_eq!(result.standings[0].0, "Egypt", "highest total wins even if not furthest on the A.S.T.");
    assert_eq!(result.standings[0].1, 600);
    assert_eq!(result.standings[1].0, "Crete");
}

#[test]
fn tied_totals_are_broken_by_furthest_ast_position() {
    let mut app = setup_app();
    let (behind, _, _) = setup_player(&mut app, "Egypt", GameFaction::Egypt);
    app.world_mut().entity_mut(behind).insert(AstPosition::new(1));
    add_treasury_tokens(&mut app, behind, 300); // 100 + 300 = 400

    let (ahead, _, _) = setup_player(&mut app, "Crete", GameFaction::Crete);
    app.world_mut().entity_mut(ahead).insert(AstPosition::new(2));
    add_treasury_tokens(&mut app, ahead, 200); // 200 + 200 = 400, same total

    app.update();

    let result = app.world().resource::<GameResult>();
    assert_eq!(result.standings[0].1, result.standings[1].1, "totals should be tied at 400");
    assert_eq!(
        result.standings[0].0, "Crete",
        "tiebreaker (35.2): furthest along the A.S.T. wins"
    );
}
