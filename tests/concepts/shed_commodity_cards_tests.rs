use adv_civ::GameActivity;
use adv_civ::civilization::{
    AwaitingCommodityShedSelection, CivilizationTradeCards, CommodityShedSelectionState,
    NeedsToShedCommodityCards, PlayerTradeCards, TradeCard, TradeCardTrait, apply_commodity_shed,
    start_shed_commodity_cards,
};
use adv_civ::stupid_ai::IsHuman;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Entity, Name, NextState, World};

fn base_world() -> World {
    let mut world = World::new();
    world.init_resource::<CommodityShedSelectionState>();
    world.init_resource::<CivilizationTradeCards>();
    world.init_resource::<NextState<GameActivity>>();
    world
}

fn run_shed_systems(world: &mut World) {
    world.run_system_once(start_shed_commodity_cards).unwrap();
    world.flush();
    world.run_system_once(apply_commodity_shed).unwrap();
    world.flush();
}

/// Nobody is over the 8-card limit: the phase should skip straight through.
#[test]
fn under_limit_player_is_untouched_and_phase_advances() {
    let mut world = base_world();

    let mut cards = PlayerTradeCards::default();
    cards.add_trade_cards(TradeCard::Ochre, 3);
    let player = world.spawn((Name::new("Player"), cards)).id();

    run_shed_systems(&mut world);

    assert!(world.get::<NeedsToShedCommodityCards>(player).is_none());
    let state = world.get_resource::<NextState<GameActivity>>().unwrap();
    assert!(matches!(state, NextState::Pending(GameActivity::ResolveCalamities)));
}

/// Regression guard for the "favor high value and series" request: given a
/// 5-card Ochre (value 1) set plus four singleton commodities of higher face
/// value, the AI must discard the cheapest-to-give singleton rather than
/// breaking the set, even though the set's individual cards are worth less
/// per-card than some of the singletons.
#[test]
fn ai_over_limit_breaks_a_singleton_before_touching_a_series() {
    let mut world = base_world();

    let mut cards = PlayerTradeCards::default();
    cards.add_trade_cards(TradeCard::Ochre, 5); // value 1, big set: costly to break (value_of_giving = 1*9 = 9)
    cards.add_trade_cards(TradeCard::Hides, 1); // value 1, singleton: cheapest to give (value_of_giving = 1*1 = 1)
    cards.add_trade_cards(TradeCard::Iron, 1); // value 2, singleton
    cards.add_trade_cards(TradeCard::Papyrus, 1); // value 2, singleton
    cards.add_trade_cards(TradeCard::Salt, 1); // value 3, singleton
    // Total 9 commodity cards, 1 over the 8-card limit.
    let ai_player: Entity = world.spawn((Name::new("AI Player"), cards)).id();

    run_shed_systems(&mut world);

    assert!(
        world.get::<NeedsToShedCommodityCards>(ai_player).is_none(),
        "the AI should have resolved its own shed immediately, no UI involved"
    );

    let hand = world.get::<PlayerTradeCards>(ai_player).unwrap();
    assert_eq!(
        hand.commodity_cards().get(&TradeCard::Ochre).copied(),
        Some(5),
        "the 5-card Ochre set must survive intact"
    );
    assert_eq!(
        hand.commodity_cards().get(&TradeCard::Hides),
        None,
        "the cheapest singleton (Hides) is what gets discarded"
    );
    assert_eq!(hand.commodity_cards().get(&TradeCard::Iron).copied(), Some(1));
    assert_eq!(
        hand.commodity_cards().get(&TradeCard::Papyrus).copied(),
        Some(1)
    );
    assert_eq!(hand.commodity_cards().get(&TradeCard::Salt).copied(), Some(1));

    let piles = world.get_resource::<CivilizationTradeCards>().unwrap();
    assert!(
        piles
            .card_piles
            .get(&TradeCard::Hides.value())
            .is_some_and(|p| p.contains(&TradeCard::Hides)),
        "the discarded card must return to its value pile"
    );
}

/// The human player must not have cards auto-discarded -- they get a UI gate
/// (`AwaitingCommodityShedSelection` + populated `CommodityShedSelectionState`)
/// instead, and stay untouched until they confirm.
#[test]
fn human_over_limit_is_gated_behind_ui_selection() {
    let mut world = base_world();

    let mut cards = PlayerTradeCards::default();
    cards.add_trade_cards(TradeCard::Ochre, 6);
    cards.add_trade_cards(TradeCard::Iron, 3);
    let human = world.spawn((Name::new("Human"), IsHuman, cards)).id();

    run_shed_systems(&mut world);

    assert!(
        world.get::<NeedsToShedCommodityCards>(human).is_some(),
        "the human must still be marked as needing to shed"
    );
    assert!(
        world
            .get::<AwaitingCommodityShedSelection>(human)
            .is_some(),
        "the human must be gated behind the selection UI"
    );

    let hand = world.get::<PlayerTradeCards>(human).unwrap();
    assert_eq!(
        hand.commodity_cards().values().sum::<usize>(),
        9,
        "nothing auto-discarded yet"
    );

    let shed_state = world.get_resource::<CommodityShedSelectionState>().unwrap();
    assert_eq!(shed_state.player, Some(human));
    assert_eq!(shed_state.must_discard, 1);
}

/// Once the UI clears `AwaitingCommodityShedSelection` (simulating the human
/// pressing Confirm after selecting cards), the confirmed discard must be
/// applied and the gating state cleared.
#[test]
fn human_confirmed_selection_is_applied() {
    let mut world = base_world();

    let mut cards = PlayerTradeCards::default();
    cards.add_trade_cards(TradeCard::Ochre, 6);
    cards.add_trade_cards(TradeCard::Iron, 3);
    let human = world.spawn((Name::new("Human"), IsHuman, cards)).id();

    // First pass: sets up the UI gate.
    run_shed_systems(&mut world);

    // Simulate the human choosing to discard 1 Iron and confirming.
    world
        .resource_mut::<CommodityShedSelectionState>()
        .increment(TradeCard::Iron);
    world
        .entity_mut(human)
        .remove::<AwaitingCommodityShedSelection>();

    // Second pass: applies the confirmed selection (start_shed_commodity_cards
    // is a no-op here since the player is no longer over the limit's trigger
    // point -- NeedsToShedCommodityCards is already attached).
    world.run_system_once(apply_commodity_shed).unwrap();
    world.flush();

    assert!(world.get::<NeedsToShedCommodityCards>(human).is_none());
    let hand = world.get::<PlayerTradeCards>(human).unwrap();
    assert_eq!(hand.commodity_cards().get(&TradeCard::Iron).copied(), Some(2));
    assert_eq!(hand.commodity_cards().get(&TradeCard::Ochre).copied(), Some(6));

    let shed_state = world.get_resource::<CommodityShedSelectionState>().unwrap();
    assert!(
        shed_state.player.is_none(),
        "selection state must be cleared after applying"
    );
}
