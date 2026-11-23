//! Calamity defence: what a civilization card is worth as insurance.
//!
//! Self-play (scripts/selfplay.py, 9 AIs, 30 rounds) showed calamities
//! taking about 80 cities a game, more than conflict and city support
//! together, while the AI bought cards with no regard for them. Every
//! personality now values the cards that soften the calamities that
//! actually hurt, scaled by how many cities it has at stake.
//!
//! The per-card values follow the cities each calamity cost per game in that
//! baseline (Superstition ~18, Slave Revolt ~7, Epidemic ~4, Earthquake ~4,
//! Civil War ~2, Famine ~2, Flood ~1, Civil Disorder ~1) times how much of
//! the damage the card removes (rules 30.x, see resolve_calamities_systems).
//! Cards that make a calamity worse (Mining: Slave Revolt; Road Building:
//! Epidemic, Civil Disorder, Iconoclasm; Military: Civil Disorder) count
//! slightly against.

use crate::civilization::CivCardName;

/// Insurance value of `card` at full exposure, before scaling by cities.
pub fn calamity_defence_value(card: CivCardName) -> f32 {
    use CivCardName::{
        Deism, Democracy, DramaAndPoetry, Engineering, Enlightenment, Law, Medicine, Military,
        Mining, Monotheism, Music, Mysticism, Philosophy, Pottery, RoadBuilding, Theology,
    };
    match card {
        // Superstition: 3 cities reduced; Mysticism saves 1, Deism 2,
        // Enlightenment all 3 (and halves Slave Revolt, 30.423).
        Mysticism => 0.30,
        Deism => 0.35,
        Enlightenment => 0.45,
        // Epidemic.
        Medicine => 0.15,
        // Earthquake (destroy becomes reduce, no secondary) and Flood.
        Engineering => 0.15,
        // Civil War selection, Iconoclasm and Heresy limit.
        Philosophy => 0.12,
        // Famine.
        Pottery => 0.08,
        // Civil War and Civil Disorder.
        Music | DramaAndPoetry | Democracy => 0.08,
        // Civil Disorder and Iconoclasm and Heresy.
        Law => 0.06,
        // Iconoclasm and Heresy.
        Monotheism | Theology => 0.03,
        Mining => -0.08,
        RoadBuilding => -0.05,
        Military => -0.03,
        _ => 0.0,
    }
}

/// Bonus for buying `card` with `cities` on the board. Even a player with no
/// city yet gets part of it: cities are coming, and the cheap insurance cards
/// are cheapest to pick up early. `risk` (0..1, personality) makes a cautious
/// player value insurance a bit more.
pub fn calamity_defence_bonus(card: CivCardName, cities: usize, risk: f32) -> f32 {
    let exposure = (cities as f32 / 3.0).clamp(0.3, 1.0);
    let caution = 1.2 - 0.4 * risk.clamp(0.0, 1.0);
    calamity_defence_value(card) * exposure * caution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superstition_insurance_beats_minor_cover() {
        assert!(
            calamity_defence_value(CivCardName::Mysticism)
                > calamity_defence_value(CivCardName::Pottery)
        );
        assert!(
            calamity_defence_value(CivCardName::Enlightenment)
                > calamity_defence_value(CivCardName::Mysticism)
        );
    }

    #[test]
    fn more_cities_more_reason_to_insure() {
        let few = calamity_defence_bonus(CivCardName::Mysticism, 0, 0.5);
        let many = calamity_defence_bonus(CivCardName::Mysticism, 4, 0.5);
        assert!(few > 0.0 && many > few);
        assert_eq!(calamity_defence_bonus(CivCardName::Mysticism, 3, 0.5), many);
    }

    #[test]
    fn cards_that_worsen_calamities_count_against() {
        assert!(calamity_defence_bonus(CivCardName::Mining, 3, 0.5) < 0.0);
        assert_eq!(calamity_defence_bonus(CivCardName::Astronomy, 3, 0.5), 0.0);
    }
}
