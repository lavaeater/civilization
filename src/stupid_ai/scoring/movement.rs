use super::{enemy_pressure, saturating, AreaSummary};
use crate::civilization::GameMove;
use crate::stupid_ai::Weights;
use bevy::platform::collections::HashMap;
use bevy::prelude::Entity;

/// Score a single movement-phase move for `player`. Higher = more attractive.
///
/// The drivers (see the design doc):
/// - reach toward city-capable, unowned land (`expansion` + `city_income`),
/// - don't abandon city squares we hold, and don't strip a source the enemy can hit
///   (`defense`),
/// - take favourable attacks / cities (`aggression`, gated by `risk`), veto clearly
///   losing attacks.
pub fn score_movement(
    mv: &GameMove,
    player: Entity,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    match mv {
        GameMove::EndMovement => score_end_movement(player, areas, w),
        GameMove::Movement(m) | GameMove::ShipFerry(m) => {
            let is_ferry = matches!(mv, GameMove::ShipFerry(_));
            score_peaceful_move(m.source, m.target, m.max_tokens, is_ferry, areas, w)
        }
        GameMove::AttackArea(m) | GameMove::AttackCity(m) => {
            let takes_city = matches!(mv, GameMove::AttackCity(_));
            score_attack(m.source, m.target, m.max_tokens, takes_city, areas, w)
        }
        // Not a movement-phase move; never selected here.
        _ => f32::NEG_INFINITY,
    }
}

fn area(areas: &HashMap<Entity, AreaSummary>, e: Entity) -> Option<&AreaSummary> {
    areas.get(&e)
}

/// The "do nothing / hold position" baseline. Its value is mostly the *defensive*
/// worth of staying put: holding city squares and not exposing positions. A modest
/// floor so the AI doesn't shuffle tokens around for no gain.
fn score_end_movement(
    player: Entity,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    let mut hold_value = 0.15; // small floor: standing pat is a legitimate option
    for (entity, s) in areas {
        if s.my_pop == 0 {
            continue;
        }
        if s.city_is_mine {
            // Holding a city is valuable, more so when threatened.
            let pressure = enemy_pressure(*entity, areas) as f32;
            hold_value += w.city_income * 0.5 + w.defense * saturating(pressure, 4.0) * 0.5;
        }
    }
    let _ = player;
    hold_value
}

fn score_peaceful_move(
    source: Entity,
    target: Entity,
    max_tokens: usize,
    is_ferry: bool,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    let (Some(src), Some(tgt)) = (area(areas, source), area(areas, target)) else {
        return 0.0;
    };

    // A move carrying zero tokens brings no population anywhere -- it's not
    // expansion, just repositioning (a ship with nothing aboard). Score it flat
    // at the neutral floor so it never outbids `EndMovement`'s 0.15 baseline;
    // otherwise the "grabbing new territory" and ferry bonuses below apply
    // regardless of cargo, and since neither area involved has our population,
    // the same false bonus fires symmetrically in both directions -- an empty
    // ship can oscillate between two connected areas forever (this tripped the
    // 120-selection movement loop guard in practice).
    if max_tokens == 0 {
        return 0.0;
    }

    let mut score = 0.0;

    // Reaching territory: grabbing new land is expansion; reinforcing my own is
    // weaker but consolidates.
    let grabbing_new = tgt.my_pop == 0;
    let capacity = saturating(tgt.max_population as f32, 4.0);
    if grabbing_new {
        score += w.expansion * (0.4 + 0.6 * capacity);
    } else {
        score += w.growth * 0.25 * capacity;
    }

    // City potential of the destination.
    if tgt.supports_city() {
        score += w.city_income * (0.5 + 0.5 * capacity);
    }

    // Ferries open up otherwise-unreachable coasts.
    if is_ferry {
        score += w.expansion * 0.3;
    }

    // Defensive cost: what we leave behind at the source.
    let remaining = src.my_pop.saturating_sub(max_tokens);
    let pressure = enemy_pressure(source, areas) as f32;
    if pressure > 0.0 {
        // Stripping a pressured area is bad; abandoning a held city is worse.
        let exposure = saturating(pressure, 4.0);
        let thinness = if remaining == 0 { 1.0 } else { 1.0 / remaining as f32 };
        let mut penalty = w.defense * exposure * thinness;
        if src.city_is_mine && remaining == 0 {
            penalty += w.defense * 0.8; // walked away from our own city
        }
        score -= penalty;
    }

    clamp01_soft(score)
}

fn score_attack(
    source: Entity,
    target: Entity,
    max_tokens: usize,
    takes_city: bool,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    let (Some(src), Some(tgt)) = (area(areas, source), area(areas, target)) else {
        return 0.0;
    };

    // Combat is attrition: roughly equal numbers trade off, so the edge is
    // attackers minus defenders. Keep one token back (the AI does this too).
    let attackers = max_tokens.saturating_sub(1).max(1) as f32;
    let defenders = tgt.enemy_pop.max(1) as f32;
    let edge = attackers - defenders; // >0 favourable

    // A wash or a loss (edge <= 0) is a bad trade — near-zero veto unless we're
    // reckless. This also covers the lone-token attack (1 vs ≥1) that would
    // otherwise tempt an aggressive AI into a pointless, repeatable suicide.
    if edge <= 0.0 && w.risk < 0.5 {
        return 0.02 * w.aggression;
    }

    let favourable = saturating(edge.max(0.0), 2.0); // [0,1)
    let even_or_worse = if edge <= 0.0 { w.risk } else { 1.0 };

    let mut score = w.aggression * (0.4 + 0.6 * favourable) * even_or_worse;

    // Taking a city is the win engine — almost everyone values it, scaled by how
    // winnable it is.
    if takes_city && edge >= 0.0 {
        score += (w.aggression * 0.5 + w.city_income) * (0.5 + 0.5 * favourable);
    }

    // Weakening a neighbour that pressures us has defensive value too.
    if tgt.supports_city() {
        score += w.aggression * 0.2;
    }

    // Cost of denuding the source to mount the attack.
    let remaining = src.my_pop.saturating_sub(max_tokens.saturating_sub(1));
    let pressure = enemy_pressure(source, areas) as f32;
    if remaining == 0 && pressure > 0.0 {
        score -= w.defense * saturating(pressure, 4.0);
    }

    clamp01_soft(score)
}

/// Keep scores in a bounded, comparable range. Negatives (heavy defensive
/// penalties) are preserved so a bad move can fall below the neutral baseline;
/// both pickers handle negative scores fine.
fn clamp01_soft(v: f32) -> f32 {
    v.clamp(-1.0, 1.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::{GameMove, MovementMove};
    use crate::stupid_ai::{Personality, Playstyle};

    fn e(i: u32) -> Entity {
        Entity::from_raw_u32(i).unwrap()
    }

    /// Two areas: `home` (a city I own) and `enemy` (a weakly held enemy city I can
    /// take). Returns (player, home, enemy, areas).
    fn winnable_city_world() -> (Entity, Entity, Entity, HashMap<Entity, AreaSummary>) {
        let player = e(1);
        let home = e(10);
        let enemy = e(20);
        let mut areas = HashMap::default();
        areas.insert(
            home,
            AreaSummary {
                max_population: 8,
                is_city_site: true,
                has_city: true,
                city_is_mine: true,
                my_pop: 6,
                enemy_pop: 0,
                neighbours: vec![enemy],
            },
        );
        areas.insert(
            enemy,
            AreaSummary {
                max_population: 8,
                is_city_site: true,
                has_city: true,
                city_is_mine: false,
                my_pop: 0,
                enemy_pop: 1, // poorly defended city
                neighbours: vec![home],
            },
        );
        (player, home, enemy, areas)
    }

    #[test]
    fn warlord_prefers_taking_a_winnable_city_over_holding() {
        let (player, home, enemy, areas) = winnable_city_world();
        let w = Personality::from_playstyle(Playstyle::Warlord).weights;

        let attack = GameMove::AttackCity(MovementMove::new(home, enemy, player, 5));
        let hold = GameMove::EndMovement;

        let attack_score = score_movement(&attack, player, &areas, &w);
        let hold_score = score_movement(&hold, player, &areas, &w);
        assert!(
            attack_score > hold_score,
            "warlord should take the city: attack {attack_score} vs hold {hold_score}"
        );
    }

    #[test]
    fn cautious_ai_vetoes_a_losing_attack() {
        let player = e(1);
        let home = e(10);
        let enemy = e(20);
        let mut areas = HashMap::default();
        areas.insert(
            home,
            AreaSummary {
                max_population: 8,
                my_pop: 2,
                neighbours: vec![enemy],
                ..Default::default()
            },
        );
        areas.insert(
            enemy,
            AreaSummary {
                max_population: 8,
                has_city: true,
                enemy_pop: 9, // hugely outnumbers us
                neighbours: vec![home],
                ..Default::default()
            },
        );
        // Turtle: low risk -> the losing attack should be vetoed near zero and
        // rank below simply ending the turn.
        let w = Personality::from_playstyle(Playstyle::Turtle).weights;
        let attack = GameMove::AttackCity(MovementMove::new(home, enemy, player, 2));
        let attack_score = score_movement(&attack, player, &areas, &w);
        let hold_score = score_movement(&GameMove::EndMovement, player, &areas, &w);
        assert!(
            attack_score < hold_score,
            "turtle should not throw tokens at a lost cause: attack {attack_score} vs hold {hold_score}"
        );
    }

    /// Regression test: an empty ferry (0 tokens aboard) between two areas
    /// neither of which the player occupies must never outscore `EndMovement`
    /// -- previously it did (both the "grabbing new territory" and ferry
    /// bonuses ignored cargo), which let an empty ship oscillate between the
    /// same two sea areas forever and trip the movement loop guard.
    #[test]
    fn empty_ferry_never_outscores_ending_movement() {
        let player = e(1);
        let home = e(10);
        let empty_coast = e(20);
        let mut areas = HashMap::default();
        areas.insert(
            home,
            AreaSummary {
                max_population: 8,
                is_city_site: true,
                neighbours: vec![empty_coast],
                ..Default::default()
            },
        );
        areas.insert(
            empty_coast,
            AreaSummary {
                max_population: 8,
                is_city_site: true,
                neighbours: vec![home],
                ..Default::default()
            },
        );
        for playstyle in [
            Playstyle::Warlord,
            Playstyle::Turtle,
            Playstyle::Expansionist,
        ] {
            let w = Personality::from_playstyle(playstyle).weights;
            let empty_ferry = GameMove::ShipFerry(MovementMove::new(home, empty_coast, player, 0));
            let ferry_score = score_movement(&empty_ferry, player, &areas, &w);
            let hold_score = score_movement(&GameMove::EndMovement, player, &areas, &w);
            assert!(
                ferry_score <= hold_score,
                "{playstyle:?}: empty ferry ({ferry_score}) should never outscore EndMovement ({hold_score})"
            );
        }
    }

    #[test]
    fn expansionist_values_grabbing_empty_city_land() {
        let player = e(1);
        let home = e(10);
        let empty = e(30);
        let mut areas = HashMap::default();
        areas.insert(
            home,
            AreaSummary {
                max_population: 5,
                my_pop: 4,
                neighbours: vec![empty],
                ..Default::default()
            },
        );
        areas.insert(
            empty,
            AreaSummary {
                max_population: 8,
                is_city_site: true,
                neighbours: vec![home],
                ..Default::default()
            },
        );
        let w = Personality::from_playstyle(Playstyle::Expansionist).weights;
        let grab = GameMove::Movement(MovementMove::new(home, empty, player, 2));
        let grab_score = score_movement(&grab, player, &areas, &w);
        let hold_score = score_movement(&GameMove::EndMovement, player, &areas, &w);
        assert!(
            grab_score > hold_score,
            "expansionist should move into the empty city site: grab {grab_score} vs hold {hold_score}"
        );
    }
}
