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

/// One open offer an AI means to post: two cards it names and gives, and the
/// two it asks for (rule 28.4: both sides name two cards truthfully; one more
/// card each way stays hidden).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiOfferPlan {
    pub give: HashMap<TradeCard, usize>,
    pub want: HashMap<TradeCard, usize>,
    /// Stack value the AI expects to gain, net of what it gives (hidden card
    /// included, valued as its cheapest remaining card). Negative for an
    /// offer made to get rid of a calamity.
    pub gain: i32,
}

/// Up to `max_offers` offers worth posting, best first.
///
/// Each asks for two *different* cards that extend sets the AI is building:
/// a partner then only has to part with two odd cards, which most hands can
/// do cheaply. (Asking for two of the same card, as before, needed a partner
/// holding a pair it was willing to break, so almost no offer was ever
/// taken.) What it gives is whatever costs it least, never a card it asks
/// for. Offers that would not leave the AI better off are not made.
pub fn plan_ai_offers(holdings: &HashMap<TradeCard, usize>, max_offers: usize) -> Vec<AiOfferPlan> {
    plan_offers(holdings, max_offers, None)
}

/// Offers for an AI holding a tradable calamity, which goes out as a hidden
/// card. Two commodities to name are enough, and it accepts losing up to
/// `allowance` stack value on them: keeping the calamity costs more.
pub fn plan_ai_dump_offers(
    holdings: &HashMap<TradeCard, usize>,
    max_offers: usize,
    allowance: i32,
) -> Vec<AiOfferPlan> {
    plan_offers(holdings, max_offers, Some(allowance))
}

fn plan_offers(
    holdings: &HashMap<TradeCard, usize>,
    max_offers: usize,
    dump_allowance: Option<i32>,
) -> Vec<AiOfferPlan> {
    let held: usize = holdings
        .iter()
        .filter(|(c, _)| c.is_commodity())
        .map(|(_, n)| *n)
        .sum();
    // Cards it pays from its commodities: the calamity is the third.
    let paid = if dump_allowance.is_some() { 2 } else { 3 };
    if held < paid || max_offers == 0 {
        return Vec::new();
    }
    // Asking for cards above our own best face value just gets ignored at the
    // table (nobody trades Gold for Ochre), so stay within reach.
    let max_own_value = holdings
        .iter()
        .filter(|(c, n)| **n > 0 && c.is_commodity())
        .map(|(c, _)| c.value())
        .max()
        .unwrap_or(0);
    let mut wants: Vec<(TradeCard, usize)> = TradeCard::iter()
        .filter(|c| c.is_commodity() && c.value() <= max_own_value)
        .map(|c| {
            (
                c,
                value_of_receiving(c, holdings.get(&c).copied().unwrap_or(0)),
            )
        })
        .collect();
    wants.sort_by_key(|(c, gain)| (std::cmp::Reverse(*gain), format!("{c:?}")));
    wants.truncate(5);

    let mut plans: Vec<AiOfferPlan> = wants
        .iter()
        .tuple_combinations()
        .filter_map(|((a, gain_a), (b, gain_b))| {
            let rest: HashMap<TradeCard, usize> = holdings
                .iter()
                .filter(|(c, _)| *c != a && *c != b)
                .map(|(c, n)| (*c, *n))
                .collect();
            let give = cheapest_cards_to_give(&rest, paid);
            if give.values().sum::<usize>() < paid {
                return None;
            }
            let cost = giving_cost(&rest, &give);
            // The third (hidden) card is the one cheapest_cards_to_give picked
            // last; name the other two.
            let named = cheapest_cards_to_give(&rest, 2);
            let gain = (gain_a + gain_b) as i32 - cost as i32;
            if gain <= -dump_allowance.unwrap_or(0) {
                return None;
            }
            Some(AiOfferPlan {
                give: named,
                want: HashMap::from([(*a, 1), (*b, 1)]),
                gain,
            })
        })
        .collect();
    plans.sort_by_key(|p| std::cmp::Reverse(p.gain));
    plans.truncate(max_offers);
    plans
}

/// Stack value lost by giving `give` out of `holdings`.
fn giving_cost(holdings: &HashMap<TradeCard, usize>, give: &HashMap<TradeCard, usize>) -> usize {
    give.iter()
        .map(|(card, n)| {
            let have = holdings.get(card).copied().unwrap_or(0);
            let left = have.saturating_sub(*n);
            card.value() * (have * have - left * left)
        })
        .sum()
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

/// Stack value of a hand of commodities: each set of `n` cards of face value
/// `v` is worth `v·n²` (rule 28.2).
pub fn hand_value(holdings: &HashMap<TradeCard, usize>) -> usize {
    holdings
        .iter()
        .filter(|(card, _)| card.is_commodity())
        .map(|(card, n)| card.value() * n * n)
        .sum()
}

/// What an offer does to the offerer's hand, as far as it can be known when
/// it is composed: the named cards leave, the hidden cards it gives are taken
/// to be its cheapest commodities, and the named cards it asks for arrive.
/// The hidden cards it gets back are unknown and left out.
/// Returns (value now, value after).
pub fn offer_hand_values(
    holdings: &HashMap<TradeCard, usize>,
    giving: &HashMap<TradeCard, usize>,
    hidden_giving: usize,
    receiving: &HashMap<TradeCard, usize>,
) -> (usize, usize) {
    let before = hand_value(holdings);
    let mut after = holdings.clone();
    for (card, n) in giving {
        if let Some(have) = after.get_mut(card) {
            *have = have.saturating_sub(*n);
        }
    }
    for (card, n) in cheapest_cards_to_give(&after, hidden_giving) {
        if let Some(have) = after.get_mut(&card) {
            *have = have.saturating_sub(n);
        }
    }
    for (card, n) in receiving {
        *after.entry(*card).or_insert(0) += n;
    }
    (before, hand_value(&after))
}

#[cfg(test)]
mod offer_plan_tests {
    use super::*;

    fn hand(cards: &[(TradeCard, usize)]) -> HashMap<TradeCard, usize> {
        cards.iter().copied().collect()
    }

    #[test]
    fn asks_for_two_different_cards_and_never_gives_them() {
        let h = hand(&[
            (TradeCard::Ochre, 3),
            (TradeCard::Iron, 2),
            (TradeCard::Hides, 1),
            (TradeCard::Papyrus, 1),
            (TradeCard::Salt, 1),
        ]);
        let plans = plan_ai_offers(&h, 3);
        let best = plans.first().expect("this hand has offers worth making");
        for p in &plans {
            assert_eq!(p.want.len(), 2, "two different cards: {p:?}");
            assert_eq!(p.give.values().sum::<usize>(), 2);
            assert!(p.give.keys().all(|c| !p.want.contains_key(c)));
            assert!(p.gain > 0);
        }
        // Best offer extends the two biggest sets.
        assert!(
            best.want.contains_key(&TradeCard::Ochre) || best.want.contains_key(&TradeCard::Iron)
        );
    }

    #[test]
    fn a_calamity_goes_out_with_two_commodities_at_a_loss() {
        // Two Grain: giving both costs 2*4=8, receiving two odd cards gains 2.
        let h = hand(&[(TradeCard::Grain, 2)]);
        assert_eq!(plan_ai_offers(&h, 3), vec![]);
        let plans = plan_ai_dump_offers(&h, 3, 10);
        assert_ne!(plans, vec![]);
        assert!(plans.iter().all(|p| p.gain < 0 && p.gain > -10));
        assert_eq!(plan_ai_dump_offers(&h, 3, 4), vec![]);
    }

    #[test]
    fn no_offer_without_three_cards_to_trade() {
        assert_eq!(plan_ai_offers(&hand(&[(TradeCard::Ochre, 2)]), 3), vec![]);
    }

    #[test]
    fn does_not_break_a_big_set_for_singletons() {
        // Only Grain to give: losing three of a 4-set (cost 4*(16-1)=60) is
        // never worth two singletons.
        let h = hand(&[(TradeCard::Grain, 4)]);
        assert_eq!(plan_ai_offers(&h, 3), vec![]);
    }

    #[test]
    fn an_offer_shows_what_it_does_to_the_hand() {
        let hand = HashMap::from([
            (TradeCard::Ochre, 3),
            (TradeCard::Iron, 1),
            (TradeCard::Salt, 1),
        ]);
        // Give Iron and Salt, hide one more card (an Ochre is cheapest),
        // get two Ochre back: 3 Ochre + Iron + Salt -> 4 Ochre.
        let giving = HashMap::from([(TradeCard::Iron, 1), (TradeCard::Salt, 1)]);
        let receiving = HashMap::from([(TradeCard::Ochre, 2)]);
        let (before, after) = offer_hand_values(&hand, &giving, 1, &receiving);
        assert_eq!(before, 9 + 2 + 3);
        assert_eq!(after, 16);
    }
}
