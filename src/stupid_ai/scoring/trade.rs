//! Personality knobs for the (countdown-driven) AI trade systems in
//! `concepts/trade`. Trade is hidden-information and multi-step, so rather than a
//! single move-score we expose small decision helpers the existing `ai_*` trade
//! systems consult. See `docs/utility-ai-design.md` §4 (Trade).

use crate::stupid_ai::Weights;

/// Extra stack-value gain (beyond break-even) an AI demands before accepting an
/// offer. Eager traders (high `trade_drive`) take marginal trades; reluctant ones
/// hold out for a clear profit.
pub fn trade_accept_margin(w: &Weights) -> f32 {
    // trade_drive 1.0 -> 0 margin (accept any non-losing trade)
    // trade_drive 0.0 -> require +3 stack value
    (1.0 - w.trade_drive) * 3.0
}

/// Whether this AI bothers to take a trade purely to offload a calamity card.
/// Calamity-averse players jump at it; the carefree ignore the risk.
pub fn accepts_calamity_offload(w: &Weights) -> bool {
    w.calamity_aversion >= 0.4
}

/// Probability [0,1] that the AI creates a fresh offer this tick. Merchants flood
/// the table with offers; turtles/warlords mostly sit out the trade phase.
pub fn offer_creation_chance(w: &Weights) -> f32 {
    (0.25 + 0.75 * w.trade_drive).clamp(0.0, 1.0)
}

/// Trade-phase countdown value below which this AI stops trading. The phase counts
/// down from 90; a higher threshold = stops sooner. Eager traders keep going.
/// AIs post offers once a second, so even the most reluctant stays for a few
/// rounds of offers (it used to leave after 2s, before the first one).
pub fn stop_trading_threshold(w: &Weights) -> f32 {
    // trade_drive 1.0 -> 74 (trade for ~16s)
    // trade_drive 0.0 -> 86 (stay ~4s)
    86.0 - 12.0 * w.trade_drive
}

// ── Bluffing and trust (rule 28.3) ───────────────────────────────────────
//
// Each side of a deal names two of its cards truthfully; everything else
// "may be any commodity or tradable calamity card, regardless of what was
// said". So an offer can carry a *claim* about its hidden cards ("...and two
// more Salt") that is nothing but talk. Honest AIs claim what they will
// really hand over; a deceptive one holding a calamity claims whatever sells
// and slips the calamity in. Every AI remembers, per partner, how well that
// partner's word has held up, and values claims (and fears hidden cards) by it.

/// How willing this personality is to lie about its hidden cards: the
/// reckless and the aggressive bluff, the cautious rarely do.
pub fn deceit(w: &Weights) -> f32 {
    (0.6 * w.risk + 0.4 * w.aggression).clamp(0.0, 1.0)
}

/// Hidden cards an AI puts in its offer (and asks for in return). One by
/// default; one more to hide a tradable calamity among, and one more when its
/// hand is poor and it is willing to gamble -- a weak hand has little to lose
/// and a bigger deal is the fastest way to a set.
pub fn hidden_cards_to_trade(
    holds_tradable_calamity: bool,
    hand_value: usize,
    w: &Weights,
) -> usize {
    let mut hidden = 1;
    if holds_tradable_calamity {
        hidden += 1;
    }
    if hand_value < 15 && w.risk >= 0.5 {
        hidden += 1;
    }
    hidden
}

/// Trust in a partner nobody has traded with yet.
pub const DEFAULT_TRUST: f32 = 0.6;

/// Stack-value hit a hidden card that turns out to be a calamity is treated
/// as: Treachery alone costs a city.
pub const HIDDEN_CALAMITY_COST: f32 = 8.0;

/// Per-partner trust, 0 (always lies) to 1 (always honest).
#[derive(bevy::prelude::Component, Clone, Debug, Default)]
pub struct TradeTrust {
    trust: bevy::platform::collections::HashMap<bevy::prelude::Entity, f32>,
}

impl TradeTrust {
    pub fn of(&self, partner: bevy::prelude::Entity) -> f32 {
        self.trust.get(&partner).copied().unwrap_or(DEFAULT_TRUST)
    }

    /// Blend in how honest the last deal with `partner` was (0..1).
    pub fn observe(&mut self, partner: bevy::prelude::Entity, honesty: f32) {
        let t = self.of(partner);
        self.trust.insert(
            partner,
            (0.7 * t + 0.3 * honesty.clamp(0.0, 1.0)).clamp(0.0, 1.0),
        );
    }
}

/// How honest a deal looked from the receiving side: the share of claimed
/// hidden cards that actually arrived, and 0 if a calamity came with it.
pub fn honesty_of_delivery(
    claimed_hidden: &bevy::platform::collections::HashMap<crate::civilization::TradeCard, usize>,
    guaranteed: &bevy::platform::collections::HashMap<crate::civilization::TradeCard, usize>,
    received: &bevy::platform::collections::HashMap<crate::civilization::TradeCard, usize>,
) -> f32 {
    use crate::civilization::TradeCardTrait;
    if received.iter().any(|(c, n)| *n > 0 && c.is_calamity()) {
        return 0.0;
    }
    let claimed: usize = claimed_hidden.values().sum();
    if claimed == 0 {
        return 1.0;
    }
    let kept: usize = claimed_hidden
        .iter()
        .map(|(card, n)| {
            let extra = received
                .get(card)
                .copied()
                .unwrap_or(0)
                .saturating_sub(guaranteed.get(card).copied().unwrap_or(0));
            extra.min(*n)
        })
        .sum();
    kept as f32 / claimed as f32
}

/// What slipping a traded Treachery to `victim` is worth to the trader.
/// Rule 30.221: the trader takes one of the victim's cities, so a victim
/// without cities is worth nothing. Taking a city from the player with the
/// most cities also holds the leader back, and a partner who burnt us
/// before is the one to pay back. Aggressive players value it most.
pub fn treachery_victim_value(
    victim_cities: usize,
    most_cities: usize,
    trust_in_victim: f32,
    w: &Weights,
) -> f32 {
    if victim_cities == 0 {
        return 0.0;
    }
    let city = 6.0 * (0.6 + 0.4 * w.aggression.clamp(0.0, 1.0));
    let leader = if victim_cities >= most_cities {
        2.0
    } else {
        0.0
    };
    let payback = 3.0 * ((DEFAULT_TRUST - trust_in_victim) / DEFAULT_TRUST).clamp(0.0, 1.0);
    city + leader + payback
}

/// The best player to aim a Treachery at among `others` (player, cities).
pub fn best_treachery_victim(
    others: &[(bevy::prelude::Entity, usize)],
    trust: Option<&TradeTrust>,
    w: &Weights,
) -> Option<bevy::prelude::Entity> {
    let most = others.iter().map(|(_, c)| *c).max().unwrap_or(0);
    others
        .iter()
        .map(|(p, cities)| {
            let t = trust.map_or(DEFAULT_TRUST, |t| t.of(*p));
            (*p, treachery_victim_value(*cities, most, t, w))
        })
        .filter(|(_, v)| *v > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(p, _)| p)
}

/// How much an AI wants a calamity it directs to hit `victim` (Piracy 30.912,
/// Iconoclasm 30.818). It gains nothing itself, so what counts is holding back
/// whoever is ahead on cities and paying back a past liar. Only the ranking
/// matters: rule 29.64 says the effects must be directed at someone.
pub fn harm_victim_value(
    victim_cities: usize,
    most_cities: usize,
    trust_in_victim: f32,
    w: &Weights,
) -> f32 {
    let ahead = victim_cities as f32 / most_cities.max(1) as f32;
    let payback = ((DEFAULT_TRUST - trust_in_victim) / DEFAULT_TRUST).clamp(0.0, 1.0);
    ahead + (0.5 + 0.5 * w.aggression.clamp(0.0, 1.0)) * payback
}

#[cfg(test)]
mod treachery_tests {
    use super::*;
    use crate::stupid_ai::{Personality, Playstyle};
    use bevy::prelude::Entity;

    fn e(i: u32) -> Entity {
        Entity::from_raw_u32(i).unwrap()
    }

    #[test]
    fn treachery_goes_to_the_leader_not_the_cityless() {
        let w = Personality::from_playstyle(Playstyle::Balanced).weights;
        let others = [(e(1), 0), (e(2), 2), (e(3), 5)];
        assert_eq!(best_treachery_victim(&others, None, &w), Some(e(3)));
        assert_eq!(best_treachery_victim(&[(e(1), 0)], None, &w), None);
    }

    #[test]
    fn a_liar_gets_paid_back() {
        let w = Personality::from_playstyle(Playstyle::Balanced).weights;
        let mut trust = TradeTrust::default();
        for _ in 0..5 {
            trust.observe(e(2), 0.0);
        }
        // Equal cities: the one who burnt us gets it.
        let others = [(e(1), 3), (e(2), 3)];
        assert_eq!(best_treachery_victim(&others, Some(&trust), &w), Some(e(2)));
    }

    #[test]
    fn harm_goes_to_the_leader_or_a_liar() {
        let w = Personality::from_playstyle(Playstyle::Balanced).weights;
        let honest = DEFAULT_TRUST;
        assert!(harm_victim_value(5, 5, honest, &w) > harm_victim_value(2, 5, honest, &w));
        assert!(harm_victim_value(4, 5, 0.0, &w) > harm_victim_value(5, 5, honest, &w));
        assert!(harm_victim_value(0, 0, honest, &w).is_finite());
    }
}

#[cfg(test)]
mod bluff_tests {
    use super::*;
    use crate::civilization::TradeCard;
    use crate::stupid_ai::{Personality, Playstyle};
    use bevy::platform::collections::HashMap;
    use bevy::prelude::Entity;

    #[test]
    fn warlords_bluff_more_than_turtles() {
        let warlord = Personality::from_playstyle(Playstyle::Warlord).weights;
        let turtle = Personality::from_playstyle(Playstyle::Turtle).weights;
        assert!(deceit(&warlord) > deceit(&turtle));
    }

    #[test]
    fn calamities_and_poor_hands_make_bigger_deals() {
        let w = Weights::uniform(0.6);
        assert_eq!(hidden_cards_to_trade(false, 40, &w), 1);
        assert_eq!(hidden_cards_to_trade(true, 40, &w), 2);
        assert_eq!(hidden_cards_to_trade(true, 5, &w), 3);
        assert_eq!(hidden_cards_to_trade(false, 5, &Weights::uniform(0.2)), 1);
    }

    #[test]
    fn a_lie_or_a_calamity_costs_trust() {
        let p = Entity::from_raw_u32(5).unwrap();
        let salt2 = HashMap::from([(TradeCard::Salt, 2)]);
        let named = HashMap::from([(TradeCard::Iron, 1), (TradeCard::Ochre, 1)]);
        // Promised 2 more Salt, got 1 Salt and a Hides.
        let got = HashMap::from([
            (TradeCard::Iron, 1),
            (TradeCard::Ochre, 1),
            (TradeCard::Salt, 1),
            (TradeCard::Hides, 1),
        ]);
        assert!((honesty_of_delivery(&salt2, &named, &got) - 0.5).abs() < 1e-6);
        let burnt = HashMap::from([
            (TradeCard::Iron, 1),
            (TradeCard::Ochre, 1),
            (TradeCard::Treachery, 1),
        ]);
        assert_eq!(
            honesty_of_delivery(&HashMap::default(), &named, &burnt),
            0.0
        );

        let mut trust = TradeTrust::default();
        trust.observe(p, 0.0);
        assert!(trust.of(p) < DEFAULT_TRUST);
        let low = trust.of(p);
        trust.observe(p, 1.0);
        assert!(trust.of(p) > low);
    }
}
