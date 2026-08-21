use adv_civ::civilization::{
    CivilizationTradeCards, MIN_CARDS_REQUIRED_TO_TRADE, NINTH_STACK_COST, PlayerTradeCards,
    TokenStock, TradeCard, Treasury, buy_from_ninth_stack,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::World;
use pretty_assertions::assert_eq;

#[test]
fn test_can_trade() {
    let mut ptc = PlayerTradeCards::default();
    for _ in 0..=MIN_CARDS_REQUIRED_TO_TRADE {
        ptc.add_trade_card(TradeCard::Ochre);
    }
    assert!(ptc.can_trade());
    let mut ptc = PlayerTradeCards::default();
    for _ in 0..(MIN_CARDS_REQUIRED_TO_TRADE - 1) {
        ptc.add_trade_card(TradeCard::Ochre);
    }
    assert!(!ptc.can_trade());
}

#[test]
fn test_add_and_has_trade_card() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Salt);
    assert!(ptc.has_trade_card(TradeCard::Salt));
}

#[test]
fn test_remove_n_trade_cards() {
    let mut ptc = PlayerTradeCards::default();
    for _ in 0..3 {
        ptc.add_trade_card(TradeCard::Iron);
    }
    let removed = ptc.remove_n_trade_cards(2, TradeCard::Iron);
    assert_eq!(removed.unwrap(), 2);
    assert_eq!(ptc.number_of_trade_cards(), 1);
}

#[test]
fn test_add_trade_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_cards(TradeCard::Wine, 2);
    assert_eq!(ptc.number_of_cards_for_trade_card(TradeCard::Wine), 2);
}

#[test]
fn test_number_of_tradeable_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Papyrus);
    ptc.add_trade_card(TradeCard::VolcanoEarthquake);
    assert_eq!(ptc.number_of_tradeable_cards(), 1);
}

#[test]
fn test_number_of_trade_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Gold);
    ptc.add_trade_card(TradeCard::Piracy);
    assert_eq!(ptc.number_of_trade_cards(), 2);
}

#[test]
fn test_has_n_commodities() {
    let mut ptc = PlayerTradeCards::default();
    for _ in 0..3 {
        ptc.add_trade_card(TradeCard::Ivory);
    }
    assert!(ptc.has_n_of_card(3, TradeCard::Ivory));
    assert!(!ptc.has_n_of_card(4, TradeCard::Ivory));
}

#[test]
fn test_cards_of_commodity_type() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Dye);
    ptc.add_trade_card(TradeCard::Dye);
    let cards = ptc.number_of_cards_for_trade_card(TradeCard::Dye);
    assert_eq!(cards, 2);
}

#[test]
fn test_remove_card_for_calamity() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Flood);
    let removed = ptc.remove_n_trade_cards(1, TradeCard::Flood).unwrap();
    assert_eq!(removed, 1);
}

#[test]
fn test_remove_n_of_commodity() {
    let mut ptc = PlayerTradeCards::default();
    for _ in 0..2 {
        ptc.add_trade_card(TradeCard::Timber);
    }
    let removed = ptc.remove_n_trade_cards(2, TradeCard::Timber);
    assert_eq!(removed.unwrap(), 2);
}

#[test]
fn test_remove_worst_commodity() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Bronze);
    let removed = ptc.remove_worst_commodity();
    assert!(removed.is_some());
}

#[test]
fn test_remove_worst_tradeable_calamity() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Piracy);
    let removed = ptc.remove_worst_tradeable_calamity();
    assert_eq!(removed, Some(TradeCard::Piracy));
}

#[test]
fn test_trade_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Spices);
    assert_eq!(ptc.number_of_trade_cards(), 1);
}

#[test]
fn test_calamity_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::CivilWar);
    let set = ptc.calamity_cards();
    assert!(set.contains(&TradeCard::CivilWar));
}

#[test]
fn test_commodity_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Resin);
    let set = ptc.commodity_cards();
    assert!(set.contains_key(&TradeCard::Resin));
}

#[test]
fn test_commodities() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Gold);
    let set = ptc.commodities();
    assert!(set.contains(&TradeCard::Gold));
}

#[test]
fn test_number_of_cards_of_commodity() {
    let mut ptc = PlayerTradeCards::default();
    for _ in 0..2 {
        ptc.add_trade_card(TradeCard::Papyrus);
    }
    assert_eq!(ptc.number_of_cards_for_trade_card(TradeCard::Papyrus), 2);
}

#[test]
fn test_has_trade_cards() {
    let mut ptc = PlayerTradeCards::default();
    assert!(!ptc.has_trade_cards());
    ptc.add_trade_card(TradeCard::Ochre);
    assert!(ptc.has_trade_cards());
}

#[test]
fn test_trade_cards_grouped_by_value() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Wine);
    ptc.add_trade_card(TradeCard::Dye);
    let grouped = ptc.trade_cards_grouped_by_value();
    assert!(grouped.get(&5).is_some());
    assert!(grouped.get(&8).is_some());
}

#[test]
fn test_trade_cards_grouped_by_value_and_type() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::Wine);
    ptc.add_trade_card(TradeCard::Dye);
    ptc.add_trade_card(TradeCard::BarbarianHordes);
    let grouped = ptc.trade_cards_grouped_by_value();
    assert!(grouped.get(&5).is_some());
    assert!(grouped.get(&5).unwrap().get(&TradeCard::Wine).is_some());
    assert!(grouped.get(&8).unwrap().get(&TradeCard::Dye).is_some());
    assert!(
        grouped
            .get(&5)
            .unwrap()
            .get(&TradeCard::BarbarianHordes)
            .is_some()
    );
}

#[test]
fn test_number_of_tradeable_calamity_cards() {
    let mut ptc = PlayerTradeCards::default();
    ptc.add_trade_card(TradeCard::SlaveRevolt);
    ptc.add_trade_card(TradeCard::VolcanoEarthquake);
    assert_eq!(ptc.number_of_tradeable_calamity_cards(), 1);
}

#[test]
fn test_commodity_card_suites() {
    let mut ptc = PlayerTradeCards::default();
    // Add 3 Resin cards with value 2 each
    ptc.add_trade_card(TradeCard::Resin);
    ptc.add_trade_card(TradeCard::Resin);
    ptc.add_trade_card(TradeCard::Resin);
    // Add 2 Salt cards with value 1 each
    ptc.add_trade_card(TradeCard::Salt);
    ptc.add_trade_card(TradeCard::Salt);
    // Add 1 Timber card with value 3
    ptc.add_trade_card(TradeCard::Gold);

    let suites = ptc.commodity_card_suites();
    // (3 Resin)^2 * 2 = 9 * 7 = 63
    assert_eq!(suites.get(&TradeCard::Resin), Some(&63));
    // (2 Salt)^2 * 3 = 4 * 3 = 12
    assert_eq!(suites.get(&TradeCard::Salt), Some(&12));
    // (1 Timber)^2 * 9 = 1 * 9 = 9
    assert_eq!(suites.get(&TradeCard::Gold), Some(&9));
    // No other commodities present
    assert_eq!(suites.len(), 3);
}

// ── Rule 27.51: buying from the ninth (Gold/Ivory/Piracy) stack ────────────

fn treasury_with_tokens(world: &mut World, count: usize) -> Treasury {
    let mut treasury = Treasury::default();
    for _ in 0..count {
        treasury.add_token_to_treasury(world.spawn_empty().id());
    }
    treasury
}

#[test]
fn buying_charges_18_tokens_per_card_and_returns_them_to_stock() {
    let mut world = World::new();
    let mut treasury = treasury_with_tokens(&mut world, 2 * NINTH_STACK_COST);
    let mut token_stock = TokenStock::new(0, vec![]);
    let mut resource = CivilizationTradeCards {
        card_piles: HashMap::from([(9, vec![TradeCard::Gold, TradeCard::Ivory])]),
    };
    let mut hand = PlayerTradeCards::default();

    let bought = buy_from_ninth_stack(&mut treasury, &mut token_stock, &mut resource, &mut hand, 2);

    assert_eq!(bought, 2);
    assert_eq!(treasury.tokens_in_treasury(), 0);
    assert_eq!(token_stock.tokens_in_stock(), 2 * NINTH_STACK_COST);
    assert_eq!(
        hand.number_of_cards_for_trade_card(TradeCard::Gold)
            + hand.number_of_cards_for_trade_card(TradeCard::Ivory),
        2
    );
    assert!(resource.card_piles.get(&9).unwrap().is_empty());
}

#[test]
fn insufficient_treasury_blocks_the_purchase() {
    let mut world = World::new();
    let mut treasury = treasury_with_tokens(&mut world, NINTH_STACK_COST - 1);
    let mut token_stock = TokenStock::new(0, vec![]);
    let mut resource = CivilizationTradeCards {
        card_piles: HashMap::from([(9, vec![TradeCard::Piracy])]),
    };
    let mut hand = PlayerTradeCards::default();

    let bought = buy_from_ninth_stack(&mut treasury, &mut token_stock, &mut resource, &mut hand, 1);

    assert_eq!(bought, 0);
    assert_eq!(treasury.tokens_in_treasury(), NINTH_STACK_COST - 1);
    assert_eq!(token_stock.tokens_in_stock(), 0);
    assert!(!hand.has_trade_card(TradeCard::Piracy));
    // Nothing was spent, so the stack is untouched.
    assert_eq!(resource.card_piles.get(&9).unwrap().len(), 1);
}

#[test]
fn an_empty_ninth_stack_blocks_the_purchase_even_with_plenty_of_treasury() {
    let mut world = World::new();
    let mut treasury = treasury_with_tokens(&mut world, 10 * NINTH_STACK_COST);
    let mut token_stock = TokenStock::new(0, vec![]);
    let mut resource = CivilizationTradeCards {
        card_piles: HashMap::from([(9, vec![])]),
    };
    let mut hand = PlayerTradeCards::default();

    let bought = buy_from_ninth_stack(&mut treasury, &mut token_stock, &mut resource, &mut hand, 3);

    assert_eq!(bought, 0);
    assert_eq!(treasury.tokens_in_treasury(), 10 * NINTH_STACK_COST);
}

#[test]
fn purchase_count_is_capped_by_whichever_runs_out_first_stack_or_treasury() {
    let mut world = World::new();
    // Enough treasury for 3 cards, but the stack only has 2.
    let mut treasury = treasury_with_tokens(&mut world, 3 * NINTH_STACK_COST);
    let mut token_stock = TokenStock::new(0, vec![]);
    let mut resource = CivilizationTradeCards {
        card_piles: HashMap::from([(9, vec![TradeCard::Gold, TradeCard::Ivory])]),
    };
    let mut hand = PlayerTradeCards::default();

    let bought = buy_from_ninth_stack(&mut treasury, &mut token_stock, &mut resource, &mut hand, 5);

    assert_eq!(
        bought, 2,
        "capped by the stack, not the requested max_cards"
    );
    assert_eq!(
        treasury.tokens_in_treasury(),
        NINTH_STACK_COST,
        "only 2 cards' worth spent"
    );
}
