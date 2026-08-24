use crate::civilization::concepts::acquire_trade_cards::{TradeCard, TradeCardTrait};
use bevy::platform::collections::HashMap;

/// What one more copy of `card` is worth to a player already holding `count` of
/// them.
///
/// A commodity set of `n` cards with face value `v` is worth `v·n²` (rule 28.2),
/// so the gain from the `n+1`th card is `v·((n+1)² - n²) = v·(2n+1)`. The value
/// of a commodity is therefore overwhelmingly in *concentration*: a 4th Ochre
/// (value 1) is worth 7, while a first Gem (value 5) is worth only 5. Any AI
/// that picks what to ask for by face value alone trades against itself.
pub fn value_of_receiving(card: TradeCard, count: usize) -> usize {
    card.value() * (2 * count + 1)
}

/// What giving away one copy of `card` costs a player holding `count` of them:
/// `v·(n² - (n-1)²) = v·(2n-1)`. Holding none costs nothing.
pub fn value_of_giving(card: TradeCard, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    card.value() * (2 * count - 1)
}

/// Greedily pick `wanted` individual cards to give away, cheapest first by
/// [`value_of_giving`], decrementing as it goes so the second copy of a card is
/// priced correctly. Returns card -> how many copies to hand over.
pub fn cheapest_cards_to_give(
    holdings: &HashMap<TradeCard, usize>,
    wanted: usize,
) -> HashMap<TradeCard, usize> {
    let mut remaining: HashMap<TradeCard, usize> = holdings
        .iter()
        .filter(|(card, count)| **count > 0 && card.is_commodity())
        .map(|(card, count)| (*card, *count))
        .collect();
    let mut chosen: HashMap<TradeCard, usize> = HashMap::default();

    for _ in 0..wanted {
        // Ties are broken by face value then card identity so the choice is
        // deterministic rather than dependent on hash order.
        let Some((card, _)) = remaining
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(card, count)| (*card, (value_of_giving(*card, *count), card.value())))
            .min_by_key(|(card, key)| (*key, format!("{card:?}")))
        else {
            break;
        };
        *chosen.entry(card).or_insert(0) += 1;
        if let Some(count) = remaining.get_mut(&card) {
            *count -= 1;
        }
    }
    chosen
}

/// Greedily pick `wanted` individual cards to ask for, most valuable first by
/// [`value_of_receiving`], counting each acquisition so a set the AI is already
/// building keeps climbing. `candidates` limits which commodity types are
/// plausible to request.
pub fn most_valuable_cards_to_request(
    holdings: &HashMap<TradeCard, usize>,
    candidates: &[TradeCard],
    wanted: usize,
) -> HashMap<TradeCard, usize> {
    let mut projected: HashMap<TradeCard, usize> = candidates
        .iter()
        .map(|card| (*card, holdings.get(card).copied().unwrap_or(0)))
        .collect();
    let mut chosen: HashMap<TradeCard, usize> = HashMap::default();

    for _ in 0..wanted {
        let Some(card) = projected
            .iter()
            .map(|(card, count)| (*card, value_of_receiving(*card, *count)))
            .max_by_key(|(card, gain)| (*gain, std::cmp::Reverse(format!("{card:?}"))))
            .map(|(card, _)| card)
        else {
            break;
        };
        *chosen.entry(card).or_insert(0) += 1;
        *projected.entry(card).or_insert(0) += 1;
    }
    chosen
}

use crate::civilization::concepts::acquire_trade_cards::PlayerTradeCards;
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
        >= offer.initiator_pays_guaranteed.values().sum::<usize>()
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
