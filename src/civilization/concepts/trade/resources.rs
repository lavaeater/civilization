use crate::civilization::concepts::trade::components::TradeOffer;
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use bevy::prelude::{Component, Entity, Reflect, Resource};

pub fn initiator_can_accept_trade_offer(offer: &TradeOffer, player_cards: &PlayerTradeCards) -> bool {
    // Step 1: Calculate the total number of cards required in the offer
    let total_required: usize = offer.initiator_number_of_cards();

    // Step 2: Check if the player has enough total tradeable cards
    if player_cards.number_of_tradeable_cards() < total_required {
        return false;
    }

    // Step 3: Check if the player has at least two cards of any commodity type in the offer
    let mut has_two_of_any = false;
    for (commodity, &required_quantity) in &offer.initiator_commodities {
        let player_quantity = player_cards.number_of_cards_of_commodity(commodity);
        if player_quantity >= 2 {
            has_two_of_any = true;
            break;
        } else if player_quantity >= 1 && required_quantity >= 1 {
            has_two_of_any = true;
            break;
        }
    }

    // Return true only if both conditions are satisfied
    has_two_of_any
}

pub fn receiver_can_accept_trade_offer(offer: &TradeOffer, player_cards: &PlayerTradeCards) -> bool {
    // Step 1: Calculate the total number of cards required in the offer
    let total_required: usize = offer.receiver_number_of_cards();

    // Step 2: Check if the player has enough total tradeable cards
    if player_cards.number_of_tradeable_cards() < total_required {
        return false;
    }

    // Step 3: Check if the player has at least two cards of any commodity type in the offer
    let mut has_two_of_any = false;
    for (commodity, &required_quantity) in &offer.receiver_commodities {
        let player_quantity = player_cards.number_of_cards_of_commodity(commodity);
        if player_quantity >= 2 {
            has_two_of_any = true;
            break;
        } else if player_quantity >= 1 && required_quantity >= 1 {
            has_two_of_any = true;
            break;
        }
    }

    // Return true only if both conditions are satisfied
    has_two_of_any
}

#[cfg(test)]
mod tests {
    use crate::civilization::concepts::trade::components::TradeOffer;
    use crate::civilization::concepts::trade::resources::initiator_can_accept_trade_offer;
    use crate::civilization::concepts::trade_cards::components::{PlayerTradeCards, TradeCard};
    use crate::civilization::concepts::trade_cards::enums::Calamity::BarbarianHordes;
    use crate::civilization::concepts::trade_cards::enums::Commodity::*;
    use crate::civilization::concepts::trade_cards::enums::TradeCardType::*;
    use bevy::core::Name;
    use bevy::ecs::entity::Entity;
    use bevy::utils::HashMap;
    use std::cell::RefCell;

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
        trade_offer.initiator_commodities.insert(Ochre, 2);
        trade_offer.initiator_commodities.insert(Salt, 2);
        let mut player_cards = PlayerTradeCards::default();
        player_cards.add_trade_card(TradeCard::new(1,CommodityCard(Ochre), true));
        player_cards.add_trade_card(TradeCard::new(1,CommodityCard(Ochre), true));
        player_cards.add_trade_card(TradeCard::new(5, CommodityCard(Wine), true));
        player_cards.add_trade_card(TradeCard::new(5, CalamityCard(BarbarianHordes), true));
        assert_eq!(initiator_can_accept_trade_offer(&trade_offer, &player_cards), true);
    }

    #[test]
    fn accept_trade_offer_test_fail() {
        let mut trade_offer = TradeOffer::new(create_entity(), Name::new("Initiator"));
        trade_offer.initiator_commodities.insert(Salt, 2);
        let mut player_cards = PlayerTradeCards::default();
        player_cards.add_trade_card(TradeCard::new(1, CommodityCard(Salt), true));
        player_cards.add_trade_card(TradeCard::new(5, CommodityCard(Wine), true));
        player_cards.add_trade_card(TradeCard::new(5, CalamityCard(BarbarianHordes), true));
        assert!(!initiator_can_accept_trade_offer(&trade_offer, &player_cards));
    }

    #[test]
    fn counter_trade_offer_test() {
        let initiator = create_entity();
        let receiver = create_entity();
        let mut trade_offer = TradeOffer::new(initiator, Name::new("Initiator"));
        trade_offer.receiver = Some(receiver);
        trade_offer.initiator_commodities.insert(Ochre, 2);
        trade_offer.receiver_commodities.insert(Salt, 3);

        let new_initiator = create_entity();
        let new_initiator_commodities = Some(HashMap::from([(Wine, 4)]));
        let new_receiver_commodities = Some(HashMap::from([(Ochre, 1)]));

        let counter_offer = trade_offer.counter(new_initiator, new_initiator_commodities.clone(), new_receiver_commodities.clone());

        assert_eq!(counter_offer.initiator, new_initiator);
        assert_eq!(counter_offer.receiver, Some(initiator));
        assert_eq!(counter_offer.initiator_commodities, new_initiator_commodities.unwrap());
        assert_eq!(counter_offer.receiver_commodities, new_receiver_commodities.unwrap());
        assert!(counter_offer.accepts.is_empty());
        assert!(counter_offer.rejects.is_empty());
    }

    #[test]
    fn counter_trade_offer_test_no_commodities() {
        let initiator = create_entity();
        let receiver = create_entity();
        let mut trade_offer = TradeOffer::new(initiator, Name::new("Initiator"));
        trade_offer.receiver = Some(receiver);
        trade_offer.initiator_commodities.insert(Ochre, 2);
        trade_offer.receiver_commodities.insert(Salt, 3);

        let new_initiator = create_entity();

        let counter_offer = trade_offer.counter(new_initiator, None, None);

        assert_eq!(counter_offer.initiator, new_initiator);
        assert_eq!(counter_offer.receiver, Some(initiator));
        assert_eq!(counter_offer.initiator_commodities, HashMap::from([(Salt, 3)]));
        assert_eq!(counter_offer.receiver_commodities, HashMap::from([(Ochre, 2)]));
        assert!(counter_offer.accepts.is_empty());
        assert!(counter_offer.rejects.is_empty());
    }
}

#[derive(Default, Resource)]
pub struct TradeUiState {
    pub human_player: Option<Entity>,
    pub add_offered_commodity_open: bool,
    pub add_requested_commodity_open: bool,
}