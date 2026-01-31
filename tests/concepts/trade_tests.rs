use crate::setup_player;
use adv_civ::civilization::concepts::acquire_trade_cards::trade_card_components::PlayerTradeCards;
use adv_civ::civilization::concepts::acquire_trade_cards::trade_card_enums::TradeCard;
use adv_civ::civilization::concepts::trade::trade_components::{CanTrade, InSettlement, PlayerSettlements, PlayerTradeInterests, PublishedOffer, TradeOffer};
use adv_civ::civilization::concepts::trade::trade_systems::{begin_trade_settlement, recalculate_trade_moves_for_player};
use adv_civ::civilization::enums::GameFaction;
use adv_civ::civilization::game_moves::game_moves_components::{AvailableMoves, Move as GameMove, TradeMove};
use adv_civ::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use adv_civ::{GameActivity, GameState};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use std::collections::VecDeque;

pub fn setup_trade_test_app() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .insert_state(GameState::Playing)
        .add_message::<RecalculatePlayerMoves>()
        .add_sub_state::<GameActivity>();

    // Setup player one (Egypt)
    let (player_one, _, _) = setup_player(&mut app, "player one", GameFaction::Egypt);
    let mut player_one_trade_cards = PlayerTradeCards::default();
    player_one_trade_cards.add_trade_cards(TradeCard::Salt, 4);
    player_one_trade_cards.add_trade_cards(TradeCard::Silver, 2);
    player_one_trade_cards.add_trade_cards(TradeCard::Wine, 2);
    player_one_trade_cards.add_trade_cards(TradeCard::Ochre, 4);
    player_one_trade_cards.add_trade_cards(TradeCard::Hides, 2);
    player_one_trade_cards.add_trade_cards(TradeCard::Papyrus, 2);

    app.world_mut().entity_mut(player_one).insert((
        PlayerTradeInterests::from_trade_cards(&player_one_trade_cards),
        player_one_trade_cards,
        CanTrade,
    ));

    // Setup player two (Crete)
    let (player_two, _, _) = setup_player(&mut app, "player two", GameFaction::Crete);
    let mut player_two_trade_cards = PlayerTradeCards::default();
    player_two_trade_cards.add_trade_cards(TradeCard::Salt, 2);
    player_two_trade_cards.add_trade_cards(TradeCard::Silver, 4);
    player_two_trade_cards.add_trade_cards(TradeCard::Ochre, 2);
    player_two_trade_cards.add_trade_cards(TradeCard::Hides, 4);
    player_two_trade_cards.add_trade_cards(TradeCard::Papyrus, 5);

    app.world_mut().entity_mut(player_two).insert((
        PlayerTradeInterests::from_trade_cards(&player_two_trade_cards),
        player_two_trade_cards,
        CanTrade,
    ));

    (app, player_one, player_two)
}

fn get_trade_moves(app: &mut App, player: Entity) -> HashMap<usize, GameMove> {
    app.update();
    let player_moves = app.world().entity(player).get::<AvailableMoves>();
    assert!(player_moves.is_some(), "No available moves found for player");
    player_moves.unwrap().moves.clone()
}

#[test]
fn a_simple_trade_move_is_created() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, recalculate_trade_moves_for_player);

    // On move recalculation for player one
    let mut events = app.world_mut().resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.send(RecalculatePlayerMoves::new(player_one));

    // Get and verify trade moves
    let moves = get_trade_moves(&mut app, player_one);
    assert_eq!(moves.len(), 2, "Expected 2 trade moves (propose or stop trading)");

    for (_, p_move) in moves.iter() {
        if let GameMove::Trade(TradeMove::ProposeTrade(to, cards)) = p_move {
            assert_eq!(*to, player_two, "Proposed trade should be to player two");
            assert!(!cards.is_empty(), "Trade proposal should include cards");
        } else if let GameMove::Trade(TradeMove::StopTrading) = p_move {
            // This is valid
        } else {
            panic!("Unexpected trade move type: {:?}", p_move);
        }
    }
}

#[test]
fn trade_accept_moves_are_created() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, recalculate_trade_moves_for_player);

    // Create a trade offer from player_one to player_two
    let mut trade_offer = create_trade_offer(player_one, player_two);

    trade_offer.accept(&player_two);

    let trade_entity = app.world_mut().spawn((trade_offer, PublishedOffer)).id();

    // On move recalculation for player_two (the receiver)
    let mut events = app.world_mut().resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.send(RecalculatePlayerMoves::new(player_two));

    // Get and verify trade moves
    let moves = get_trade_moves(&mut app, player_two);
    let mut found_accept = false;

    for (_, p_move) in moves.iter() {
        if let GameMove::Trade(TradeMove::AcceptOrDeclineTrade(offer_entity)) = p_move {
            assert_eq!(*offer_entity, trade_entity, "Trade offer entity mismatch");
            found_accept = true;
        }
    }

    assert!(found_accept, "Expected accept trade move");
}

#[test]
fn trade_settlement_moves_are_created() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, recalculate_trade_moves_for_player);

    // Create a trade offer from player_one to player_two
    let mut trade_offer = create_trade_offer(player_one, player_two);
    
    trade_offer.accept(&player_two);

    let trade_entity = app.world_mut().spawn((trade_offer, PublishedOffer, InSettlement)).id();

    // Add player settlements
    app.world_mut().entity_mut(player_one)
        .insert(PlayerSettlements { trades: VecDeque::from(vec![trade_entity]),
            current_trade: None });
    app.world_mut().entity_mut(player_two)
        .insert(PlayerSettlements { trades: VecDeque::from(vec![trade_entity]),
            current_trade: None });

    // On move recalculation for player_two (the receiver)
    let mut events = app.world_mut().resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.send(RecalculatePlayerMoves::new(player_two));

    // Get and verify trade moves
    let moves = get_trade_moves(&mut app, player_two);
    let mut found_settle = false;

    for (_, p_move) in moves.iter() {
        if let GameMove::Trade(TradeMove::SettleTrade(offer_entity)) = p_move {
            assert_eq!(*offer_entity, trade_entity, "Trade offer entity mismatch");
            found_settle = true;
        }
    }

    assert!(found_settle, "Expected settle trade move");
}

fn create_trade_offer(player_one: Entity, player_two: Entity) -> TradeOffer {
    let mut trade_offer = TradeOffer::propose_trade(
        player_one,
        "player_one",
        player_two,
        "player_two"
    );
    trade_offer.pay_even_more(TradeCard::Silver, 2);
    trade_offer.pay_even_more(TradeCard::Hides, 1);
    trade_offer.get_even_more(TradeCard::Salt, 2);
    trade_offer.get_even_more(TradeCard::Ochre, 1);
    trade_offer
}

#[test]
fn no_trade_moves_when_cannot_trade() {
    let (mut app, player_one, _) = setup_trade_test_app();
    app.add_systems(Update, recalculate_trade_moves_for_player);

    // Remove CanTrade component
    app.world_mut().entity_mut(player_one).remove::<CanTrade>();

    // On move recalculation
    let mut events = app.world_mut().resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.send(RecalculatePlayerMoves::new(player_one));

    // Verify no moves were created
    app.update();
    let player_moves = app.world().entity(player_one).get::<AvailableMoves>();
    assert!(player_moves.is_none() || player_moves.unwrap().moves.is_empty(),
            "No trade moves should be available when player cannot trade");
}

#[test]
fn test_begin_trade_settlement_basic() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, begin_trade_settlement);

    // Create an accepted trade offer
    let mut trade_offer = create_trade_offer(player_one, player_two);
    trade_offer.accept(&player_two);
    
    // Spawn the trade offer
    let trade_entity = app.world_mut().spawn((trade_offer, PublishedOffer)).id();
    
    // Run the system
    app.update();
    
    // Verify the trade was marked as InSettlement
    let trade = app.world().entity(trade_entity);
    assert!(trade.get::<InSettlement>().is_some(), "Trade should be marked as InSettlement");
    // 
    // // Verify both players have the trade in their settlement queue
    // for &player in &[player_one, player_two] {
    //     let settlements = app.world().entity(player).get::<PlayerSettlements>().unwrap();
    //     assert_eq!(settlements.trades.len(), 1, "Player should have 1 trade in settlement");
    //     assert_eq!(settlements.trades[0], trade_entity, "Trade entity should match");
    //     assert!(settlements.current_trade.is_none(), "Current trade should be None");
    // }
}

#[test]
fn test_begin_trade_settlement_existing_settlements() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, begin_trade_settlement);

    // Create player settlements with existing trades
    let existing_trade = app.world_mut().spawn_empty().id();
    let settlements = PlayerSettlements {
        trades: VecDeque::from(vec![existing_trade]),
        current_trade: None,
    };
    
    app.world_mut().entity_mut(player_one).insert(settlements.clone());
    app.world_mut().entity_mut(player_two).insert(settlements);

    // Create and accept a new trade
    let mut trade_offer = create_trade_offer(player_one, player_two);
    trade_offer.accept(&player_two);
    
    let new_trade_entity = app.world_mut().spawn((trade_offer, PublishedOffer)).id();
    
    // Run the system
    app.update();
    
    // Verify the trade was added to existing settlements
    for &player in &[player_one, player_two] {
        let settlements = app.world().entity(player).get::<PlayerSettlements>().unwrap();
        assert_eq!(settlements.trades.len(), 2, "Player should have 2 trades in settlement");
        assert_eq!(settlements.trades[1], new_trade_entity, "New trade should be at the end of the queue");
    }
}

#[test]
fn test_begin_trade_settlement_only_processes_accepted() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, begin_trade_settlement);

    // Create a non-accepted trade offer
    let trade_offer = create_trade_offer(player_one, player_two);
    
    let trade_entity = app.world_mut().spawn((trade_offer, PublishedOffer)).id();
    
    // Run the system
    app.update();
    
    // Verify the trade was not marked as InSettlement
    let trade = app.world().entity(trade_entity);
    assert!(trade.get::<InSettlement>().is_none(), "Non-accepted trade should not be in settlement");
    
    // Verify no settlements were created
    for &player in &[player_one, player_two] {
        let settlements = app.world().entity(player).get::<PlayerSettlements>();
        assert!(settlements.is_none(), "No settlements should be created for non-accepted trades");
    }
}

#[test]
fn test_begin_trade_settlement_does_not_process_already_settled() {
    let (mut app, player_one, player_two) = setup_trade_test_app();
    app.add_systems(Update, begin_trade_settlement);

    // Create an accepted trade offer that's already in settlement
    let mut trade_offer = create_trade_offer(player_one, player_two);
    trade_offer.accept(&player_two);
    
    let trade_entity = app.world_mut().spawn((trade_offer, PublishedOffer, InSettlement)).id();
    
    // Run the system
    app.update();
    
    // Verify the trade is still only in settlement once
    let trade = app.world().entity(trade_entity);
    assert!(trade.get::<InSettlement>().is_some(), "Trade should still be in settlement");
    
    // Verify settlements were not modified (should still be empty)
    for &player in &[player_one, player_two] {
        let settlements = app.world().entity(player).get::<PlayerSettlements>();
        assert!(settlements.is_none(), "No new settlements should be created for already settled trades");
    }
}

#[test]
fn stop_trading_move_is_available() {
    let (mut app, player_one, _) = setup_trade_test_app();
    app.add_systems(Update, recalculate_trade_moves_for_player);

    // On move recalculation
    let mut events = app.world_mut().resource_mut::<Messages<RecalculatePlayerMoves>>();
    events.send(RecalculatePlayerMoves::new(player_one));

    // Get and verify stop trading move exists
    let moves = get_trade_moves(&mut app, player_one);
    let has_stop_trading = moves.values().any(|m|
        matches!(m, GameMove::Trade(TradeMove::StopTrading))
    );

    assert!(has_stop_trading, "Expected stop trading move to be available");
}

// 
// pub struct TradeTestPlugin;
// 
// impl Plugin for TradeTestPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_message::<SendTradingCardsCommand>()
//             .add_message::<RecalculatePlayerMoves>()
//             // .add_systems(OnEnter(GameActivity::Trade), setup_trade)
//             .add_systems(
//                 Update,
//                 (
//                     button_action,
//                     // trigger_trade_moves,
//                     remove_rejected_trades,
//                     // delay_trade_moves_if_offers_are_accepted,
//                     begin_trade_settlement,
//                     handle_send_trading_cards_command,
//                     settle_trades,
//                     // check_trade_gate,
//                     recalculate_trade_moves_for_player,
//                 ),
//             )
//             .add_systems(Update, button_action)
//             // .add_observer(offer_published)
//             .add_observer(can_trade_removed);
//     }
// }
// 
// #[test]
// fn when_open_offer_available() {
//     let mut app = App::new();
//     app.add_plugins(TradeTestPlugin);
//     let (p_one, _, _) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
// 
//     let (p_two, _, _) = setup_player(&mut app, "Player 2", GameFaction::Babylon);
// 
//     let mut p_one_trading_cards = PlayerTradeCards::default();
//     p_one_trading_cards.add_trade_cards(vec![
//         TradeCard::new(1, CommodityCard(Ochre), true),
//         TradeCard::new(1, CommodityCard(Hides), true),
//         TradeCard::new(3, CommodityCard(Iron), true),
//         TradeCard::new(3, CommodityCard(Iron), true),
//         TradeCard::new(3, CommodityCard(Salt), true),
//     ]);
// 
//     let mut p_two_trading_cards = PlayerTradeCards::default();
//     p_two_trading_cards.add_trade_cards(vec![
//         TradeCard::new(1, CommodityCard(Ochre), true),
//         TradeCard::new(1, CommodityCard(Hides), true),
//         TradeCard::new(3, CommodityCard(Salt), true),
//         TradeCard::new(3, CommodityCard(Salt), true),
//         TradeCard::new(3, CommodityCard(Iron), true),
//     ]);
// 
//     // let trade_offer = TradeOffer::propose_trade_to(p_two, "Player 2", HashMap::from([(Salt,2)]));
//     //
//     // app.world_mut()
//     //     .spawn(
//     //         (
//     //             trade_offer,
//     //             PublishedOffer,
//     //         )
//     //     );
//     //
//     // app.world_mut()
//     //     .entity_mut(p_one)
//     //     .insert((p_one_trading_cards, CanTrade));
//     //
//     // app.world_mut()
//     //     .entity_mut(p_two)
//     //     .insert((p_two_trading_cards, CanTrade));
//     //
//     // let mut events = app
//     //     .world_mut()
//     //     .resource_mut::<Messages<RecalculatePlayerMoves>>();
//     //
//     // events.send(RecalculatePlayerMoves::new(p_one));
//     // // events.send(RecalculatePlayerMoves::new(p_two));
//     //
//     // // Act
//     // app.update();
//     //
//     // let p_one_moves = app.world().entity(p_one).get::<AvailableMoves>().unwrap();
//     // // let p_two_moves = app.world().entity(p_two).get::<AvailableMoves>().unwrap();
//     //
//     // print!("p_one_moves: {:#?}", p_one_moves);
//     // // print!("p_two_moves: {:#?}", p_two_moves);
// }
// 
// #[test]
// fn basic_recalc() {
//     let mut app = App::new();
//     app.add_plugins(TradeTestPlugin);
//     let (p_one, _p_one_tokens, _p_one_city_tokens) =
//         setup_player(&mut app, "Player 1", GameFaction::Egypt);
// 
//     let mut p_one_trading_cards = PlayerTradeCards::default();
//     p_one_trading_cards.add_trade_cards(vec![
//         TradeCard::new(2, CommodityCard(Ochre), true),
//         TradeCard::new(2, CommodityCard(Ochre), true),
//         TradeCard::new(3, CommodityCard(Iron), true),
//     ]);
// 
//     app.world_mut()
//         .entity_mut(p_one)
//         .insert((p_one_trading_cards, CanTrade));
// 
//     let mut events = app
//         .world_mut()
//         .resource_mut::<Messages<RecalculatePlayerMoves>>();
// 
//     let _ = events.send(RecalculatePlayerMoves::new(p_one));
// 
//     // Act
//     app.update();
// 
//     let p_one_moves = app.world().entity(p_one).get::<AvailableMoves>().unwrap();
// 
//     let second_move = p_one_moves.moves.get(&1).unwrap();
// 
//     // match &first_move {
//     //     Trade(trade_move) => {
//     //         assert_eq!(trade_move.trade_move_type, OpenTradeOffer)
//     //     }
//     //     _ => {
//     //         panic!("Expected Trade variant");
//     //     }
//     // }
// 
//     match &second_move {
//         Trade(trade_move) => {
//             assert_eq!(trade_move.trade_move_type, StopTrading)
//         }
//         _ => {
//             panic!("Expected Trade Variant");
//         }
//     }
// }
// 
// #[test]
// fn start_game() {
//     let mut app = setup_bevy_app(|mut app| {
//         app.add_message::<CheckPlayerCitySupport>()
//             .add_systems(Update, start_check_city_support);
//         app
//     });
// 
//     setup_player(&mut app, "Player 1", GameFaction::Egypt);
//     create_area(&mut app, "Egypt", 1);
// 
//     app.update();
// 
//     let state = app
//         .world()
//         .get_resource::<NextState<GameActivity>>()
//         .unwrap();
//     assert!(matches!(state, Pending(GameActivity::AcquireTradeCards)));
// }
// 
// thread_local! {
//     static ENTITY_COUNTER: RefCell<u32> = RefCell::new(0);
// }
// 
// fn create_entity() -> Entity {
//     ENTITY_COUNTER.with(|counter| {
//         let index = *counter.borrow();
//         *counter.borrow_mut() += 1; // Increment the counter for the next entity
//         Entity::from_raw(index)
//     })
// }
// 
// // #[test]
// // fn accept_trade_offer_test() {
// //     let mut trade_offer = TradeOffer::new(create_entity(), Name::new("Initiator"));
// //     trade_offer.initiator_pays_more(Ochre);
// //     trade_offer.initiator_pays_more(Ochre);
// //     trade_offer.initiator_pays_more(Salt);
// //     trade_offer.initiator_pays_more(Salt);
// //     let mut player_cards = PlayerTradeCards::default();
// //     player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Ochre), true));
// //     player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Ochre), true));
// //     player_cards.add_trade_card(TradeCard::new(5, CommodityCard(Wine), true));
// //     player_cards.add_trade_card(TradeCard::new(5, CalamityCard(BarbarianHordes), true));
// //     assert!(initiator_can_pay_for_offer(&trade_offer, &player_cards));
// // }
// //
// // #[test]
// // fn accept_trade_offer_test_fail() {
// //     let mut trade_offer = TradeOffer::new(create_entity(), Name::new("Initiator"));
// //     trade_offer.initiator_pays_more(Salt);
// //     trade_offer.initiator_pays_more(Salt);
// //     let mut player_cards = PlayerTradeCards::default();
// //     player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Salt), true));
// //     player_cards.add_trade_card(TradeCard::new(5, CommodityCard(Wine), true));
// //     player_cards.add_trade_card(TradeCard::new(5, CalamityCard(BarbarianHordes), true));
// //     assert!(!initiator_can_pay_for_offer(&trade_offer, &player_cards));
// // }
// //
// // #[test]
// // fn counter_trade_offer_test() {
// //     let initiator = create_entity();
// //     let receiver = create_entity();
// //     let mut trade_offer = TradeOffer::new(initiator, Name::new("Initiator"));
// //     trade_offer.receiver = Some(receiver);
// //     trade_offer.initiator_pays_more(Ochre);
// //     trade_offer.initiator_pays_more(Ochre);
// //     trade_offer.initiator_gets_more(Salt);
// //     trade_offer.initiator_gets_more(Salt);
// //     trade_offer.initiator_gets_more(Salt);
// //
// //     let new_initiator = create_entity();
// //     let new_initiator_commodities = HashMap::from([(Wine, 4)]);
// //     let new_receiver_commodities = HashMap::from([(Ochre, 1)]);
// //
// //     let counter_offer = trade_offer.counter(
// //         new_initiator,
// //         Some(new_initiator_commodities.clone()),
// //         Some(new_receiver_commodities.clone()),
// //     );
// //
// //     assert_eq!(counter_offer.initiator, new_initiator);
// //     assert_eq!(counter_offer.receiver, Some(initiator));
// //     assert_eq!(
// //         counter_offer.initiator_pays_guaranteed,
// //         HashMap::from([(Wine, 2)])
// //     );
// //     assert_eq!(counter_offer.initiator_pays, HashMap::from([(Wine, 2)]));
// //     assert_eq!(
// //         counter_offer.initiator_gets_guaranteed,
// //         new_receiver_commodities
// //     );
// //     assert!(counter_offer.accepts.is_empty());
// //     assert!(counter_offer.rejects.is_empty());
// // }
// //
// // #[test]
// // fn counter_trade_offer_test_no_commodities() {
// //     let initiator = create_entity();
// //     let receiver = create_entity();
// //     let mut trade_offer = TradeOffer::new(initiator, Name::new("Initiator"));
// //     trade_offer.receiver = Some(receiver);
// //     trade_offer.initiator_pays_more(Ochre);
// //     trade_offer.initiator_pays_more(Ochre);
// //     trade_offer.initiator_gets_more(Salt);
// //     trade_offer.initiator_gets_more(Salt);
// //     trade_offer.initiator_gets_more(Salt);
// //
// //     let new_initiator = create_entity();
// //
// //     let counter_offer = trade_offer.counter(new_initiator, None, None);
// //
// //     assert_eq!(counter_offer.initiator, new_initiator);
// //     assert_eq!(counter_offer.receiver, Some(initiator));
// //     assert_eq!(
// //         counter_offer.initiator_gets_guaranteed,
// //         HashMap::from([(Ochre, 2)])
// //     );
// //     assert_eq!(counter_offer.initiator_gets, HashMap::default());
// //     assert_eq!(
// //         counter_offer.initiator_pays_guaranteed,
// //         HashMap::from([(Salt, 2)])
// //     );
// //     assert_eq!(counter_offer.initiator_pays, HashMap::from([(Salt, 1)]));
// //     assert!(counter_offer.accepts.is_empty());
// //     assert!(counter_offer.rejects.is_empty());
// // }
// //
// // #[test]
// // fn worst_tradeable_calamity_test() {
// //     let mut player_cards = PlayerTradeCards::default();
// //
// //     // Initially there should be no calamity cards
// //     assert_eq!(player_cards.worst_tradeable_calamity(), None);
// //
// //     // Add some calamity cards with different values
// //     player_cards.add_trade_card(TradeCard::new(1, CalamityCard(BarbarianHordes), true));
// //     assert_eq!(
// //         player_cards.worst_tradeable_calamity().unwrap(),
// //         BarbarianHordes
// //     );
// //
// //     // Add a lower value calamity (should become the new worst)
// //     player_cards.add_trade_card(TradeCard::new(5, CalamityCard(CivilWar), true));
// //     assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), CivilWar);
// //
// //     // Add a higher value calamity (should not change the worst)
// //     player_cards.add_trade_card(TradeCard::new(3, CalamityCard(Epidemic), true));
// //     assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), CivilWar);
// //
// //     // Add a non-tradeable calamity (should not be considered)
// //     player_cards.add_trade_card(TradeCard::new(7, CalamityCard(Famine), false));
// //     assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), CivilWar);
// // }
// 
// #[test]
// fn send_trade_cards_simple() {
//     /*
//     Given a trade offer of the simplest, most basic kind, settle it.
//     Trade is 2 iron for 2 papyrus, with the hidden ones simply being ochre or hides.
//      */
//     let mut app = setup_bevy_app(|mut app| {
//         app.add_message::<SendTradingCardsCommand>()
//             .add_systems(Update, handle_send_trading_cards_command);
//         app
//     });
// 
//     let (p_one, _p_one_tokens, _p_one_city_tokens) =
//         setup_player(&mut app, "Player 1", GameFaction::Egypt);
//     let (p_two, _p_two_tokens, _p_two_city_tokens) =
//         setup_player(&mut app, "Player 2", GameFaction::Thrace);
// 
//     let mut p_one_trading_cards = PlayerTradeCards::default();
//     p_one_trading_cards.add_trade_cards(vec![
//         TradeCard::new(2, CommodityCard(Ochre), true),
//         TradeCard::new(2, CommodityCard(Ochre), true),
//         TradeCard::new(3, CommodityCard(Iron), true),
//     ]);
// 
//     let mut p_two_trading_cards = PlayerTradeCards::default();
//     p_two_trading_cards.add_trade_cards(vec![
//         TradeCard::new(2, CommodityCard(Hides), true),
//         TradeCard::new(2, CommodityCard(Hides), true),
//         TradeCard::new(3, CommodityCard(Salt), true),
//     ]);
// 
//     app.world_mut()
//         .entity_mut(p_one)
//         .insert(p_one_trading_cards);
//     app.world_mut()
//         .entity_mut(p_two)
//         .insert(p_two_trading_cards);
// 
//     let mut events = app
//         .world_mut()
//         .resource_mut::<Messages<SendTradingCardsCommand>>();
// 
//     events.send(SendTradingCardsCommand::new(
//         p_one,
//         p_two,
//         HashMap::from([(CommodityCard(Iron), 1), (CommodityCard(Ochre), 2)]),
//     ));
//     events.send(SendTradingCardsCommand::new(
//         p_two,
//         p_one,
//         HashMap::from([(CommodityCard(Salt), 1), (CommodityCard(Hides), 2)]),
//     ));
// 
//     // Act
//     app.update();
// 
//     let p_one_trading_cards = app.world().entity(p_one).get::<PlayerTradeCards>().unwrap();
//     let p_two_trading_cards = app.world().entity(p_two).get::<PlayerTradeCards>().unwrap();
//     assert_eq!(
//         p_one_trading_cards.number_of_cards_for_trade_card(&Iron),
//         0,
//         "Iron should be gone, actual: {}",
//         p_one_trading_cards.number_of_cards_for_trade_card(&Iron)
//     );
//     assert_eq!(
//         p_one_trading_cards.number_of_cards_for_trade_card(&Ochre),
//         0,
//         "Ochre should be gone, actual: {}",
//         p_one_trading_cards.number_of_cards_for_trade_card(&Ochre)
//     );
//     assert_eq!(
//         p_one_trading_cards.number_of_cards_for_trade_card(&Hides),
//         2,
//         "Hides should be 2, actual: {}",
//         p_one_trading_cards.number_of_cards_for_trade_card(&Hides)
//     );
//     assert_eq!(
//         p_one_trading_cards.number_of_cards_for_trade_card(&Salt),
//         1,
//         "Salt should be 1, actual: {}",
//         p_one_trading_cards.number_of_cards_for_trade_card(&Salt)
//     );
// 
//     assert_eq!(
//         p_two_trading_cards.number_of_cards_for_trade_card(&Iron),
//         1,
//         "Iron should be 1, actual: {}",
//         p_two_trading_cards.number_of_cards_for_trade_card(&Iron)
//     );
//     assert_eq!(
//         p_two_trading_cards.number_of_cards_for_trade_card(&Ochre),
//         2,
//         "Ochre should be 2, actual: {}",
//         p_two_trading_cards.number_of_cards_for_trade_card(&Ochre)
//     );
//     assert_eq!(
//         p_two_trading_cards.number_of_cards_for_trade_card(&Hides),
//         0,
//         "Hides should be 0, actual: {}",
//         p_two_trading_cards.number_of_cards_for_trade_card(&Hides)
//     );
//     assert_eq!(
//         p_two_trading_cards.number_of_cards_for_trade_card(&Salt),
//         0,
//         "Salt should be 0, actual: {}",
//         p_two_trading_cards.number_of_cards_for_trade_card(&Salt)
//     );
// }
