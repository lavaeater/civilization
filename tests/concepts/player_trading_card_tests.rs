use adv_civ::civilization::concepts::{
    PlayerTradeCards, TradeCard, MIN_CARDS_REQUIRED_TO_TRADE
};
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
