// ── Rule 30.312: Grain locked by a Famine reduction can't pay for a civ card ──
//
// process_civ_card_purchase is the one place every civ-card purchase actually
// commits (human UI and AI selection both funnel into ConfirmCivCardPurchase),
// so it's the authoritative enforcement point for the Grain lock -- these
// tests exercise it directly through the real ECS system, not a stand-in.

use adv_civ::civilization::{
    AvailableCivCards, CardsHeldBeforePurchasing, CivCardDefinition, CivCardName,
    CivCardSelectionState, CivCardType, CivCardsAcquisition, CivTradeUi, CivilizationTradeCards,
    ConfirmCivCardPurchase, PlayerCivilizationCards, PlayerDoneAcquiringCivilizationCards,
    PlayerTradeCards, RecalculatePlayerMoves, TokenStock, TradeCard, Treasury,
    begin_acquire_civ_cards, process_civ_card_purchase,
    resolve_calamities::resolve_calamities_components::GrainLockedForPurchase,
};
use adv_civ::player::Player;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{App, Update};

fn setup_app() -> App {
    let mut app = App::new();
    app.add_message::<ConfirmCivCardPurchase>()
        .add_message::<PlayerDoneAcquiringCivilizationCards>()
        .add_message::<RecalculatePlayerMoves>()
        .init_resource::<CivilizationTradeCards>()
        .init_resource::<CivCardSelectionState>()
        .init_resource::<AvailableCivCards>()
        .add_systems(Update, process_civ_card_purchase);
    app
}

fn spawn_player_with_grain(
    app: &mut App,
    grain_held: usize,
    grain_locked: Option<usize>,
) -> bevy::prelude::Entity {
    let mut trade_cards = PlayerTradeCards::default();
    for _ in 0..grain_held {
        trade_cards.add_trade_card(TradeCard::Grain);
    }
    let mut entity = app.world_mut().spawn((
        PlayerCivilizationCards::default(),
        trade_cards,
        Treasury::default(),
        TokenStock::new(55, Vec::new()),
    ));
    if let Some(locked) = grain_locked {
        entity.insert(GrainLockedForPurchase(locked));
    }
    entity.id()
}

#[test]
fn fully_locked_grain_cannot_be_spent_even_if_the_message_requests_it() {
    let mut app = setup_app();
    let player = spawn_player_with_grain(&mut app, 5, Some(5));

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 5);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 0,
    });
    app.update();

    // The card is still granted (this system trusts cards_to_buy), but not a
    // single locked Grain card should have left the player's hand.
    let civ_cards = app.world().get::<PlayerCivilizationCards>(player).unwrap();
    assert!(civ_cards.owns(&CivCardName::Pottery));
    let trade_cards = app.world().get::<PlayerTradeCards>(player).unwrap();
    assert_eq!(
        trade_cards.number_of_cards_for_trade_card(TradeCard::Grain),
        5
    );
}

#[test]
fn partially_locked_grain_only_the_unlocked_portion_is_spent() {
    let mut app = setup_app();
    // 5 held, 3 locked -> only 2 usable, even though the message asks for 4.
    let player = spawn_player_with_grain(&mut app, 5, Some(3));

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 4);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 0,
    });
    app.update();

    let trade_cards = app.world().get::<PlayerTradeCards>(player).unwrap();
    // Only 2 (the usable amount) were removed: 5 - 2 = 3 remain.
    assert_eq!(
        trade_cards.number_of_cards_for_trade_card(TradeCard::Grain),
        3
    );
}

#[test]
fn no_lock_component_allows_full_normal_grain_spending() {
    let mut app = setup_app();
    let player = spawn_player_with_grain(&mut app, 5, None);

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 5);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 0,
    });
    app.update();

    let trade_cards = app.world().get::<PlayerTradeCards>(player).unwrap();
    assert_eq!(
        trade_cards.number_of_cards_for_trade_card(TradeCard::Grain),
        0
    );
}

#[test]
fn grain_lock_does_not_affect_other_commodity_types_in_the_same_payment() {
    let mut app = setup_app();
    let mut trade_cards = PlayerTradeCards::default();
    for _ in 0..3 {
        trade_cards.add_trade_card(TradeCard::Grain);
    }
    for _ in 0..3 {
        trade_cards.add_trade_card(TradeCard::Salt);
    }
    let player = app
        .world_mut()
        .spawn((
            PlayerCivilizationCards::default(),
            trade_cards,
            Treasury::default(),
            TokenStock::new(55, Vec::new()),
            GrainLockedForPurchase(3),
        ))
        .id();

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 3);
    payment.insert(TradeCard::Salt, 2);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 0,
    });
    app.update();

    let trade_cards = app.world().get::<PlayerTradeCards>(player).unwrap();
    // Grain fully blocked (locked), Salt spent normally.
    assert_eq!(
        trade_cards.number_of_cards_for_trade_card(TradeCard::Grain),
        3
    );
    assert_eq!(
        trade_cards.number_of_cards_for_trade_card(TradeCard::Salt),
        1
    );
}

// ── Rule 31.1/31.4: treasury tokens help pay for civilization cards ──
//
// "Each player may acquire one or more civilization cards by turning in
// commodity cards *and treasury tokens*" -- but 31.4 bars spending more tokens
// than the purchase requires. Spent tokens go back to stock, the same route
// ninth-stack purchases use.

/// A one-card catalogue so the commit path can work out what a purchase costs.
fn app_with_card(name: CivCardName, cost: u32) -> App {
    let mut app = setup_app();
    app.insert_resource(AvailableCivCards {
        cards: vec![CivCardDefinition {
            name,
            description: String::new(),
            card_type: CivCardType::Crafts.into(),
            cost,
            credits: vec![],
            prerequisites: vec![],
        }],
    });
    app
}

fn treasury_tokens(app: &mut App, player: bevy::prelude::Entity, count: usize) {
    let tokens: Vec<bevy::prelude::Entity> = (0..count)
        .map(|_| app.world_mut().spawn_empty().id())
        .collect();
    let mut treasury = app.world_mut().get_mut::<Treasury>(player).unwrap();
    for token in tokens {
        treasury.add_token_to_treasury(token);
    }
}

#[test]
fn treasury_tokens_pay_the_part_the_cards_do_not_cover() {
    let mut app = app_with_card(CivCardName::Pottery, 10);
    let player = spawn_player_with_grain(&mut app, 1, None); // 1 Grain = 4 points
    treasury_tokens(&mut app, player, 20);

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 1);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 6,
    });
    app.update();

    let treasury = app.world().get::<Treasury>(player).unwrap();
    assert_eq!(
        treasury.tokens_in_treasury(),
        14,
        "6 tokens should be spent"
    );
    let stock = app.world().get::<TokenStock>(player).unwrap();
    assert_eq!(stock.tokens_in_stock(), 6, "spent tokens return to stock");
    assert!(
        app.world()
            .get::<PlayerCivilizationCards>(player)
            .unwrap()
            .owns(&CivCardName::Pottery)
    );
}

#[test]
fn no_more_treasury_tokens_are_spent_than_the_card_costs() {
    // Rule 31.4: a player "may not intentionally spend more treasury tokens
    // than required". The cards already cover 4 of a cost of 10, so however
    // many tokens the message asks for, only 6 may leave the treasury.
    let mut app = app_with_card(CivCardName::Pottery, 10);
    let player = spawn_player_with_grain(&mut app, 1, None);
    treasury_tokens(&mut app, player, 20);

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 1);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 20,
    });
    app.update();

    let treasury = app.world().get::<Treasury>(player).unwrap();
    assert_eq!(treasury.tokens_in_treasury(), 14);
}

#[test]
fn treasury_spending_cannot_exceed_the_treasury() {
    let mut app = app_with_card(CivCardName::Pottery, 100);
    let player = spawn_player_with_grain(&mut app, 0, None);
    treasury_tokens(&mut app, player, 3);

    app.world_mut().write_message(ConfirmCivCardPurchase {
        player,
        cards_to_buy: vec![CivCardName::Pottery],
        payment: HashMap::default(),
        treasury_tokens: 50,
    });
    app.update();

    let treasury = app.world().get::<Treasury>(player).unwrap();
    assert_eq!(treasury.tokens_in_treasury(), 0);
    let stock = app.world().get::<TokenStock>(player).unwrap();
    assert_eq!(stock.tokens_in_stock(), 3);
}

// ── An AI purchase must not disturb the human's open purchase dialog ──
//
// Every player acquires civ cards simultaneously, so AI purchases commit in the
// same frames the human is deciding. process_civ_card_purchase used to despawn
// every CivTradeUi and clear the shared selection state on any purchase, which
// wiped the human's dialog out from under them -- leaving no way to buy and the
// phase waiting forever on a player with no UI.

#[test]
fn an_ai_purchase_leaves_the_human_purchase_ui_alone() {
    let mut app = setup_app();
    let ai = spawn_player_with_grain(&mut app, 5, None);
    let human_ui = app.world_mut().spawn(CivTradeUi).id();
    let human = app.world_mut().spawn_empty().id();
    {
        let mut selection = app.world_mut().resource_mut::<CivCardSelectionState>();
        selection.player_entity = Some(human);
        selection.selected_cards.insert(CivCardName::Pottery);
    }

    let mut payment = HashMap::default();
    payment.insert(TradeCard::Grain, 2);
    app.world_mut().write_message(ConfirmCivCardPurchase {
        player: ai,
        cards_to_buy: vec![CivCardName::Pottery],
        payment,
        treasury_tokens: 0,
    });
    app.update();

    assert!(
        app.world().get_entity(human_ui).is_ok(),
        "the AI's purchase despawned the human's purchase dialog"
    );
    let selection = app.world().resource::<CivCardSelectionState>();
    assert_eq!(selection.player_entity, Some(human));
    assert!(
        selection.selected_cards.contains(&CivCardName::Pottery),
        "the AI's purchase cleared the human's card selection"
    );
}

// ── Rule 31.53: credits from a card can't be used the same turn it's bought ──
//
// begin_acquire_civ_cards snapshots each player's held cards into
// CardsHeldBeforePurchasing before this turn's purchases can grow the live
// PlayerCivilizationCards -- every credit calculation this turn (human UI,
// AI's iterative buy loop, agent API) reads that snapshot instead of the
// live hand. These tests exercise the real system directly.

fn setup_acquire_app() -> App {
    let mut app = App::new();
    app.init_resource::<CivCardsAcquisition>()
        .init_resource::<CivCardSelectionState>()
        .add_systems(Update, begin_acquire_civ_cards);
    app
}

#[test]
fn begin_acquire_snapshots_currently_held_cards() {
    let mut app = setup_acquire_app();
    let mut cards = PlayerCivilizationCards::default();
    cards.cards.insert(CivCardName::Pottery);
    cards.cards.insert(CivCardName::Mining);
    let player = app.world_mut().spawn((Player, cards)).id();

    app.update();

    let snapshot = app
        .world()
        .get::<CardsHeldBeforePurchasing>(player)
        .unwrap();
    let expected: HashSet<CivCardName> = [CivCardName::Pottery, CivCardName::Mining]
        .into_iter()
        .collect();
    assert_eq!(snapshot.0, expected);
}

#[test]
fn begin_acquire_snapshots_empty_hand_for_a_player_with_no_civ_cards_yet() {
    let mut app = setup_acquire_app();
    let player = app.world_mut().spawn(Player).id();

    app.update();

    let snapshot = app
        .world()
        .get::<CardsHeldBeforePurchasing>(player)
        .unwrap();
    assert!(snapshot.0.is_empty());
}

#[test]
fn begin_acquire_snapshot_is_frozen_even_if_the_live_hand_grows_afterward() {
    let mut app = setup_acquire_app();
    let mut cards = PlayerCivilizationCards::default();
    cards.cards.insert(CivCardName::Pottery);
    let player = app.world_mut().spawn((Player, cards)).id();

    app.update();

    // Simulate a purchase this turn adding a new card to the live hand.
    app.world_mut()
        .get_mut::<PlayerCivilizationCards>(player)
        .unwrap()
        .cards
        .insert(CivCardName::Mining);

    // The snapshot taken at the start of the turn must not have changed --
    // this is the actual guarantee rule 31.53 needs: a card bought earlier
    // this turn can't discount a card bought later the same turn.
    let snapshot = app
        .world()
        .get::<CardsHeldBeforePurchasing>(player)
        .unwrap();
    let expected: HashSet<CivCardName> = [CivCardName::Pottery].into_iter().collect();
    assert_eq!(snapshot.0, expected);
}
