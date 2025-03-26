use crate::{create_area, setup_bevy_app, setup_player};
use adv_civ::civilization::concepts::trade::components::TradeOffer;
use adv_civ::civilization::concepts::trade::events::SendTradingCardsCommand;
use adv_civ::civilization::concepts::trade::functions::initiator_can_pay_for_offer;
use adv_civ::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
use adv_civ::civilization::concepts::trade_cards::enums::Calamity::{BarbarianHordes, CivilWar, Epidemic, Famine};
use adv_civ::civilization::concepts::trade_cards::enums::Commodity::{Hides, Iron, Ochre, Salt, Wine};
use adv_civ::civilization::concepts::trade_cards::enums::TradeCardType::{
    CalamityCard, CommodityCard,
};
use adv_civ::civilization::enums::prelude::GameFaction;
use adv_civ::civilization::events::prelude::CheckPlayerCitySupport;
use adv_civ::civilization::systems::prelude::{handle_send_trading_cards_command, start_check_city_support};
use adv_civ::GameActivity;
use bevy::core::Name;
use bevy::ecs::entity::Entity;
use bevy::prelude::NextState::Pending;
use bevy::prelude::{Events, NextState, Update};
use bevy::utils::HashMap;
use std::cell::RefCell;

#[test]
fn start_game() {
    let mut app = setup_bevy_app(|mut app| {
        app.add_event::<CheckPlayerCitySupport>()
            .add_systems(Update, start_check_city_support);
        app
    });

    setup_player(&mut app, "Player 1", GameFaction::Egypt);
    create_area(&mut app, "Egypt", 1);

    app.update();

    let state = app
        .world()
        .get_resource::<NextState<GameActivity>>()
        .unwrap();
    assert!(matches!(state, Pending(GameActivity::AcquireTradeCards)));
}

thread_local! {
    static ENTITY_COUNTER: RefCell<u32> = RefCell::new(0);
}

fn create_entity() -> Entity {
    ENTITY_COUNTER.with(|counter| {
        let index = *counter.borrow();
        *counter.borrow_mut() += 1; // Increment the counter for the next entity
        Entity::from_raw(index)
    })
}

#[test]
fn accept_trade_offer_test() {
    let mut trade_offer = TradeOffer::new(create_entity(), Name::new("Initiator"));
    trade_offer.initiator_pays_more(Ochre);
    trade_offer.initiator_pays_more(Ochre);
    trade_offer.initiator_pays_more(Salt);
    trade_offer.initiator_pays_more(Salt);
    let mut player_cards = PlayerTradeCards::default();
    player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Ochre), true));
    player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Ochre), true));
    player_cards.add_trade_card(TradeCard::new(5, CommodityCard(Wine), true));
    player_cards.add_trade_card(TradeCard::new(5, CalamityCard(BarbarianHordes), true));
    assert!(initiator_can_pay_for_offer(&trade_offer, &player_cards));
}

#[test]
fn accept_trade_offer_test_fail() {
    let mut trade_offer = TradeOffer::new(create_entity(), Name::new("Initiator"));
    trade_offer.initiator_pays_more(Salt);
    trade_offer.initiator_pays_more(Salt);
    let mut player_cards = PlayerTradeCards::default();
    player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Salt), true));
    player_cards.add_trade_card(TradeCard::new(5, CommodityCard(Wine), true));
    player_cards.add_trade_card(TradeCard::new(5, CalamityCard(BarbarianHordes), true));
    assert!(!initiator_can_pay_for_offer(&trade_offer, &player_cards));
}

#[test]
fn counter_trade_offer_test() {
    let initiator = create_entity();
    let receiver = create_entity();
    let mut trade_offer = TradeOffer::new(initiator, Name::new("Initiator"));
    trade_offer.receiver = Some(receiver);
    trade_offer.initiator_pays_more(Ochre);
    trade_offer.initiator_pays_more(Ochre);
    trade_offer.initiator_gets_more(Salt);
    trade_offer.initiator_gets_more(Salt);
    trade_offer.initiator_gets_more(Salt);

    let new_initiator = create_entity();
    let new_initiator_commodities = HashMap::from([(Wine, 4)]);
    let new_receiver_commodities = HashMap::from([(Ochre, 1)]);

    let counter_offer = trade_offer.counter(
        new_initiator,
        Some(new_initiator_commodities.clone()),
        Some(new_receiver_commodities.clone()),
    );

    assert_eq!(counter_offer.initiator, new_initiator);
    assert_eq!(counter_offer.receiver, Some(initiator));
    assert_eq!(
        counter_offer.initiator_pays_guaranteed,
        HashMap::from([(Wine, 2)])
    );
    assert_eq!(
        counter_offer.initiator_pays,
        HashMap::from([(Wine, 2)])
    );
    assert_eq!(
        counter_offer.initiator_gets_guaranteed,
        new_receiver_commodities
    );
    assert!(counter_offer.accepts.is_empty());
    assert!(counter_offer.rejects.is_none());
}

#[test]
fn counter_trade_offer_test_no_commodities() {
    let initiator = create_entity();
    let receiver = create_entity();
    let mut trade_offer = TradeOffer::new(initiator, Name::new("Initiator"));
    trade_offer.receiver = Some(receiver);
    trade_offer.initiator_pays_more(Ochre);
    trade_offer.initiator_pays_more(Ochre);
    trade_offer.initiator_gets_more(Salt);
    trade_offer.initiator_gets_more(Salt);
    trade_offer.initiator_gets_more(Salt);

    let new_initiator = create_entity();

    let counter_offer = trade_offer.counter(new_initiator, None, None);

    assert_eq!(counter_offer.initiator, new_initiator);
    assert_eq!(counter_offer.receiver, Some(initiator));
    assert_eq!(counter_offer.initiator_gets_guaranteed, HashMap::from([(Ochre, 2)]));
    assert_eq!(counter_offer.initiator_gets, HashMap::default());
    assert_eq!(counter_offer.initiator_pays_guaranteed, HashMap::from([(Salt, 2)]));
    assert_eq!(counter_offer.initiator_pays,HashMap::from([(Salt, 1)]));
    assert!(counter_offer.accepts.is_empty());
    assert!(counter_offer.rejects.is_none());
}

#[test]
fn worst_tradeable_calamity_test() {
    let mut player_cards = PlayerTradeCards::default();
    
    // Initially there should be no calamity cards
    assert_eq!(player_cards.worst_tradeable_calamity(), None);
    
    // Add some calamity cards with different values
    player_cards.add_trade_card(TradeCard::new(1, CalamityCard(BarbarianHordes), true));
    assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), BarbarianHordes);
    
    // Add a lower value calamity (should become the new worst)
    player_cards.add_trade_card(TradeCard::new(5, CalamityCard(CivilWar), true));
    assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), CivilWar);
    
    // Add a higher value calamity (should not change the worst)
    player_cards.add_trade_card(TradeCard::new(3, CalamityCard(Epidemic), true));
    assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), CivilWar);
    
    // Add a non-tradeable calamity (should not be considered)
    player_cards.add_trade_card(TradeCard::new(7, CalamityCard(Famine), false));
    assert_eq!(player_cards.worst_tradeable_calamity().unwrap(), CivilWar);
}

#[test]
fn send_trade_cards_simple() {
    /*
    Given a trade offer of the simplest, most basic kind, settle it.
    Trade is 2 iron for 2 papyrus, with the hidden ones simply being ochre or hides.
     */
    let mut app = setup_bevy_app(|mut app| {
        app.add_event::<SendTradingCardsCommand>()
            .add_systems(Update, handle_send_trading_cards_command);
        app
    });

    let (p_one, _p_one_tokens, _p_one_city_tokens) = setup_player(&mut app, "Player 1", GameFaction::Egypt);
    let (p_two, _p_two_tokens, _p_two_city_tokens) = setup_player(&mut app, "Player 2", GameFaction::Thrace);
    
    let mut p_one_trading_cards = PlayerTradeCards::default();
    p_one_trading_cards.add_trade_cards( vec![TradeCard::new(2, CommodityCard(Ochre), true), TradeCard::new(2, CommodityCard(Ochre), true), TradeCard::new(3, CommodityCard(Iron), true)]);
    
    let mut p_two_trading_cards = PlayerTradeCards::default();
    p_two_trading_cards.add_trade_cards( vec![TradeCard::new(2, CommodityCard(Hides), true), TradeCard::new(2, CommodityCard(Hides), true), TradeCard::new(3, CommodityCard(Salt), true)]);

    app
        .world_mut()
        .entity_mut(p_one)
        .insert(p_one_trading_cards);
    app
        .world_mut()
        .entity_mut(p_two)
        .insert(p_two_trading_cards);

    let mut events = app.world_mut()
        .resource_mut::<Events<SendTradingCardsCommand>>();

    events.send(SendTradingCardsCommand::new(p_one, p_two, HashMap::from([(CommodityCard(Iron), 1), (CommodityCard(Ochre), 2)])));
    events.send(SendTradingCardsCommand::new(p_two, p_one, HashMap::from([(CommodityCard(Salt), 1), (CommodityCard(Hides), 2)])));

    // Act
    app.update();

    let p_one_trading_cards = app
        .world()
        .entity(p_one)
        .get::<PlayerTradeCards>().unwrap();
    let p_two_trading_cards = app
        .world()
        .entity(p_two)
        .get::<PlayerTradeCards>().unwrap();
    assert_eq!(p_one_trading_cards.number_of_cards_of_commodity(&Iron), 0, "Iron should be gone, actual: {}", p_one_trading_cards.number_of_cards_of_commodity(&Iron));
    assert_eq!(p_one_trading_cards.number_of_cards_of_commodity(&Ochre), 0, "Ochre should be gone, actual: {}", p_one_trading_cards.number_of_cards_of_commodity(&Ochre));
    assert_eq!(p_one_trading_cards.number_of_cards_of_commodity(&Hides), 2, "Hides should be 2, actual: {}", p_one_trading_cards.number_of_cards_of_commodity(&Hides));
    assert_eq!(p_one_trading_cards.number_of_cards_of_commodity(&Salt), 1, "Salt should be 1, actual: {}", p_one_trading_cards.number_of_cards_of_commodity(&Salt));

    assert_eq!(p_two_trading_cards.number_of_cards_of_commodity(&Iron), 1, "Iron should be 1, actual: {}", p_two_trading_cards.number_of_cards_of_commodity(&Iron));
    assert_eq!(p_two_trading_cards.number_of_cards_of_commodity(&Ochre), 2, "Ochre should be 2, actual: {}", p_two_trading_cards.number_of_cards_of_commodity(&Ochre));
    assert_eq!(p_two_trading_cards.number_of_cards_of_commodity(&Hides), 0, "Hides should be 0, actual: {}", p_two_trading_cards.number_of_cards_of_commodity(&Hides));
    assert_eq!(p_two_trading_cards.number_of_cards_of_commodity(&Salt), 0, "Salt should be 0, actual: {}", p_two_trading_cards.number_of_cards_of_commodity(&Salt));
}