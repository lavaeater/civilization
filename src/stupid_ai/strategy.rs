//! Win strategies: *what* an AI is trying to win by, on top of its
//! [`Playstyle`](crate::stupid_ai::Playstyle) (*how* it plays: its temperament).
//!
//! Each road maps onto a term of the rule-35 final score:
//! civ-card face value + commodity sets + treasury + **100 per A.S.T. space** +
//! **50 per city**. See `docs/utility-ai-design.md` §10.
//!
//! The strategy acts in three places:
//! - it nudges the personality's [`Weights`] once, at setup ([`Strategy::adjust`]),
//! - it adds a card-choice bonus during civ-card acquisition
//!   ([`strategy_card_bonus`], driven by A.S.T. gate needs and card affinity),
//! - it sets movement-phase pressure: a city-building floor ([`Strategy::city_floor`])
//!   and, for the Conqueror, a bonus for attacking whoever is *leading*
//!   ([`Strategy::conquest`] + [`rival_standing`]).

use crate::civilization::{AstEpoch, CivCardName, CivCardType};
use crate::stupid_ai::Weights;
use bevy::prelude::Reflect;
use enumflags2::BitFlags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum Strategy {
    /// Race the A.S.T. (100 pts/space). Meets each epoch gate as cheaply as
    /// possible: just enough cities, cheap cards in the missing colour groups.
    /// Finishing first also ends the game before rivals cash in.
    Pioneer,
    /// Maximise civ-card face value: expensive cards and credit chains. Slower
    /// on the track, but a late-game points engine.
    Scholar,
    /// Topple whoever is winning: take cities from the leader (+50 for us,
    /// -50 for them, and it can freeze their A.S.T. at an epoch gate).
    Conqueror,
    /// Cities (50 pts each, plus tax income) and commodity sets: build as many
    /// cities as the land allows and trade hard.
    Magnate,
}

impl Strategy {
    pub const ALL: [Strategy; 4] = [
        Strategy::Pioneer,
        Strategy::Scholar,
        Strategy::Conqueror,
        Strategy::Magnate,
    ];

    pub fn from_name(name: &str) -> Option<Strategy> {
        match name.trim().to_ascii_lowercase().as_str() {
            "pioneer" | "ast" | "race" => Some(Strategy::Pioneer),
            "scholar" | "tech" | "cards" => Some(Strategy::Scholar),
            "conqueror" | "conquest" | "war" => Some(Strategy::Conqueror),
            "magnate" | "cities" | "wealth" => Some(Strategy::Magnate),
            _ => None,
        }
    }

    /// One-off nudge to the playstyle's weights. Additive and clamped, so the
    /// temperament still shows through: a Turtle Conqueror only takes fights
    /// it's sure to win.
    pub fn adjust(self, mut w: Weights) -> Weights {
        let bump = |v: &mut f32, d: f32| *v = (*v + d).clamp(0.0, 1.0);
        match self {
            Strategy::Pioneer => {
                bump(&mut w.city_income, 0.15);
                bump(&mut w.tech_focus, 0.2);
                bump(&mut w.aggression, -0.1);
            }
            Strategy::Scholar => {
                bump(&mut w.tech_focus, 0.3);
                bump(&mut w.trade_drive, 0.15);
                bump(&mut w.calamity_aversion, 0.1);
            }
            Strategy::Conqueror => {
                bump(&mut w.aggression, 0.25);
                bump(&mut w.risk, 0.15);
                bump(&mut w.defense, -0.1);
            }
            Strategy::Magnate => {
                bump(&mut w.city_income, 0.25);
                bump(&mut w.trade_drive, 0.25);
                bump(&mut w.growth, 0.1);
            }
        }
        w
    }

    /// How hard to chase A.S.T. gate requirements when buying cards (0..1).
    pub fn gate_drive(self) -> f32 {
        match self {
            Strategy::Pioneer => 1.0,
            Strategy::Scholar | Strategy::Magnate => 0.5,
            Strategy::Conqueror => 0.4,
        }
    }

    /// Floor on movement/city-construction city drive, regardless of urgency.
    ///
    /// Non-zero for *every* road: civ cards are paid for with trade cards, and
    /// trade cards come from cities. In self-play, Scholars with a 0 floor
    /// held 0–1 cities and bought no cards at all, starving their own engine.
    pub fn city_floor(self) -> f32 {
        match self {
            Strategy::Magnate => 0.9,
            Strategy::Pioneer => 0.7,
            Strategy::Scholar => 0.6,
            Strategy::Conqueror => 0.4,
        }
    }

    /// Extra attack value against rivals who are ahead (0 = indifferent).
    pub fn conquest(self) -> f32 {
        match self {
            Strategy::Conqueror => 0.6,
            _ => 0.0,
        }
    }

    /// Cards that serve this road beyond their points.
    pub fn favours(self, card: CivCardName) -> bool {
        use CivCardName::{
            Agriculture, Architecture, Astronomy, ClothMaking, Coinage, Democracy, Engineering,
            Enlightenment, Law, Literacy, Mathematics, Medicine, Metalworking, Military, Mining,
            Monotheism, Philosophy, Pottery, RoadBuilding, Theology,
        };
        match self {
            // Combat, mobility and holding captured cities.
            Strategy::Conqueror => matches!(
                card,
                Metalworking | Military | RoadBuilding | Engineering | ClothMaking | Astronomy
            ),
            // City economy and calamity insurance for a wide empire.
            Strategy::Magnate => matches!(
                card,
                Coinage
                    | Mining
                    | Agriculture
                    | Pottery
                    | Architecture
                    | Medicine
                    | Law
                    | Democracy
            ),
            // Credit chains into the big point cards.
            Strategy::Scholar => matches!(
                card,
                Mathematics
                    | Literacy
                    | Philosophy
                    | Monotheism
                    | Theology
                    | Enlightenment
                    | Astronomy
                    | Medicine
            ),
            Strategy::Pioneer => false,
        }
    }

    /// Strategy a playstyle leans to when nothing else assigns one.
    pub fn natural_for(playstyle: crate::stupid_ai::Playstyle) -> Strategy {
        use crate::stupid_ai::Playstyle::{
            Balanced, Builder, Expansionist, Merchant, Turtle, Warlord,
        };
        match playstyle {
            Balanced => Strategy::Pioneer,
            Warlord => Strategy::Conqueror,
            Expansionist | Builder => Strategy::Magnate,
            Merchant | Turtle => Strategy::Scholar,
        }
    }
}

/// What still stands between a player and the next A.S.T. epoch gate ahead.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AstNeeds {
    pub cities_short: usize,
    pub missing_groups: BitFlags<CivCardType>,
    pub groups_short: usize,
    pub cards_short: usize,
}

impl AstNeeds {
    /// Needs for the first epoch after the one `space` is in. Looking one gate
    /// ahead (not just one space) lets a Pioneer buy the colour groups it'll
    /// need *before* its marker reaches the gate and freezes.
    pub fn compute(
        space: u32,
        cities: usize,
        owned_groups: BitFlags<CivCardType>,
        card_count: usize,
    ) -> Self {
        let current = AstEpoch::for_space(space);
        let next = (space + 1..space + 16)
            .map(AstEpoch::for_space)
            .find(|e| *e != current)
            .unwrap_or(current);
        let have_groups = owned_groups.iter().count();
        AstNeeds {
            cities_short: next.min_cities().saturating_sub(cities),
            missing_groups: if have_groups < next.min_card_groups() {
                BitFlags::<CivCardType>::all() & !owned_groups
            } else {
                BitFlags::empty()
            },
            groups_short: next.min_card_groups().saturating_sub(have_groups),
            cards_short: next.min_card_count().saturating_sub(card_count),
        }
    }
}

/// Strategy bonus for buying one civ card (added to `score_civ_card`).
pub fn strategy_card_bonus(
    strategy: Strategy,
    card: CivCardName,
    card_type: BitFlags<CivCardType>,
    cost: u32,
    needs: &AstNeeds,
) -> f32 {
    let mut bonus = 0.0;
    let cheapness = 1.0 - (cost as f32 / 250.0).clamp(0.0, 1.0);
    // A.S.T. gates: a card in a missing colour group, or any card while short
    // on count (cheaper = more count per commodity spent).
    if needs.groups_short > 0 && card_type.intersects(needs.missing_groups) {
        bonus += strategy.gate_drive() * 0.5;
    }
    if needs.cards_short > 0 {
        bonus += strategy.gate_drive() * 0.2 * cheapness;
    }
    match strategy {
        Strategy::Pioneer => bonus += 0.25 * cheapness,
        // Face value is score; saturate so 250-pt cards aren't infinitely better.
        Strategy::Scholar => bonus += 0.4 * (cost as f32 / (cost as f32 + 150.0)),
        Strategy::Conqueror | Strategy::Magnate => {}
    }
    if strategy.favours(card) {
        bonus += 0.35;
    }
    bonus
}

/// Rule-35 score estimate from the parts visible mid-game.
pub fn rule35_score(
    ast_space: u32,
    cities: usize,
    civ_face_value: u32,
    commodity_value: u32,
    treasury: u32,
) -> u32 {
    civ_face_value + commodity_value + treasury + ast_space * 100 + cities as u32 * 50
}

/// How far ahead `rival` is of `me` (0..1, 0.5 = level). Used to point a
/// Conqueror at the leader rather than the nearest weakling.
pub fn rival_standing(me: u32, rival: u32) -> f32 {
    let diff = rival as f32 - me as f32;
    (0.5 + 0.5 * (diff / 300.0).tanh()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stone_age_player_needs_two_cities_for_early_bronze() {
        let needs = AstNeeds::compute(2, 0, BitFlags::empty(), 0);
        assert_eq!(needs.cities_short, 2);
        assert_eq!(needs.groups_short, 0);
    }

    #[test]
    fn early_bronze_player_looks_ahead_to_late_bronze_groups() {
        let owned = CivCardType::Arts | CivCardType::Crafts;
        let needs = AstNeeds::compute(5, 3, owned, 2);
        assert_eq!(needs.cities_short, 0);
        assert_eq!(needs.groups_short, 1);
        assert!(!needs.missing_groups.contains(CivCardType::Arts));
        assert!(needs.missing_groups.contains(CivCardType::Religion));
    }

    #[test]
    fn pioneer_prefers_a_cheap_card_in_a_missing_group() {
        let needs = AstNeeds {
            groups_short: 1,
            missing_groups: BitFlags::from(CivCardType::Religion),
            ..Default::default()
        };
        let mysticism = strategy_card_bonus(
            Strategy::Pioneer,
            CivCardName::Mysticism,
            CivCardType::Religion.into(),
            50,
            &needs,
        );
        let mining = strategy_card_bonus(
            Strategy::Pioneer,
            CivCardName::Mining,
            CivCardType::Crafts.into(),
            180,
            &needs,
        );
        assert!(mysticism > mining + 0.4, "{mysticism} vs {mining}");
    }

    #[test]
    fn scholar_prefers_face_value_conqueror_prefers_military() {
        let needs = AstNeeds::default();
        let b = |s, n, c| strategy_card_bonus(s, n, CivCardType::Civics.into(), c, &needs);
        assert!(
            b(Strategy::Scholar, CivCardName::Philosophy, 240)
                > b(Strategy::Scholar, CivCardName::Pottery, 45)
        );
        assert!(
            b(Strategy::Conqueror, CivCardName::Military, 180)
                > b(Strategy::Conqueror, CivCardName::Philosophy, 240)
        );
    }

    #[test]
    fn standing_points_at_the_leader() {
        assert!(rival_standing(500, 900) > 0.75);
        assert!(rival_standing(900, 500) < 0.25);
        assert!((rival_standing(500, 500) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn adjust_is_clamped() {
        let w = Strategy::Conqueror.adjust(Weights::uniform(0.95));
        assert!(w.aggression <= 1.0 && w.risk <= 1.0);
    }
}
