use crate::civilization::concepts::acquire_trade_cards::trade_card_components::PlayerTradeCards;
use crate::civilization::concepts::trade::trade_components::TradeOffer;
use itertools::Itertools;

pub fn initiator_can_pay_for_offer(offer: &TradeOffer, initiator_cards: &PlayerTradeCards) -> bool {
    // Step 1: Calculate the total number of cards required in the offer
    let total_required: usize = offer.pays_number_of_cards();

    // Step 2: Check if the player has enough total tradeable cards
    if initiator_cards.number_of_tradeable_cards() < total_required {
        return false;
    }

    // Step 3: Check if the player has the cards that are guaranteed
    offer
        .initiator_pays_guaranteed
        .iter()
        .map(|(c, _)| initiator_cards.number_of_cards_for_trade_card(*c))
        .sum::<usize>()
        >= offer.initiator_pays_guaranteed.values().sum()
}

#[allow(dead_code)]
pub fn offer_pays_well_enough(trade_offer: &TradeOffer, trading_cards: &PlayerTradeCards) -> bool {
    let mut accept_trade = false;
    if trade_offer.initiator_pays.keys().len() >= 3 {
        if trading_cards
            .top_commodity()
            .is_some_and(|c| trade_offer.initiator_pays.keys().contains(&c))
        {
            accept_trade = true;
        } else {
            let mut matching_payment = 0;
            trade_offer
                .initiator_pays
                .iter()
                .for_each(|(commodity, _)| {
                    if !trading_cards
                        .top_commodity()
                        .is_some_and(|c| c == *commodity)
                    {
                        let score = trading_cards.number_of_cards_for_trade_card(*commodity);
                        if score > 1 {
                            matching_payment += 2;
                        } else if score > 0 {
                            matching_payment += 1;
                        }
                    }
                });
            accept_trade = matching_payment > 1;
        }
    }
    accept_trade
}

pub fn receiver_can_pay_for_offer(offer: &TradeOffer, receiver_cards: &PlayerTradeCards) -> bool {
    // Step 1: Calculate the total number of cards required in the offer
    let total_required: usize = offer.gets_number_of_cards();

    // Step 2: Check if the player has enough total tradeable cards
    if receiver_cards.number_of_tradeable_cards() < total_required {
        return false;
    }

    // Step 3: Check if the player has at least two cards of any commodity type in the offer
    offer
        .initiator_gets_guaranteed
        .iter()
        .map(|(c, _)| receiver_cards.number_of_cards_for_trade_card(*c))
        .sum::<usize>()
        >= offer.initiator_gets_guaranteed.values().sum::<usize>()
}
