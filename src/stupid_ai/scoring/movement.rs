use super::{AreaSummary, CityPlan, SiegePlan, enemy_pressure, saturating};
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
///   losing attacks,
/// - pile tokens onto a reachable city site until it can be built (`plan`), since
///   movement is the *only* way a stack reaches the build threshold.
pub fn score_movement(
    mv: &GameMove,
    player: Entity,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
    plan: &CityPlan,
    rivals: &Rivals,
) -> f32 {
    match mv {
        GameMove::EndMovement => score_end_movement(player, areas, w),
        GameMove::Movement(m) | GameMove::ShipFerry(m) => {
            let is_ferry = matches!(mv, GameMove::ShipFerry(_));
            // Ferries aren't classified as attacks, so a ferry into an enemy city
            // is really a (at most 5-token, hence always hopeless) city attack.
            if is_ferry
                && m.max_tokens > 0
                && area(areas, m.target).is_some_and(|t| t.has_city && !t.city_is_mine)
            {
                return 0.02 * w.aggression;
            }
            let base = score_peaceful_move(m.source, m.target, m.max_tokens, is_ferry, areas, w);
            if m.max_tokens == 0 {
                return base;
            }
            // Moving into an area already at its limit only feeds surplus removal
            // (the extra tokens are stripped at end of turn) -- unless it's a
            // planned city site, where going over the limit is the whole point.
            if !plan.is_target(m.target)
                && area(areas, m.target).is_some_and(|t| t.my_pop >= t.max_population)
            {
                return 0.0;
            }
            clamp01_soft(
                base + city_plan_adjustment(m.source, m.target, areas, plan)
                    + surplus_bonus(m.source, areas, plan),
            )
        }
        GameMove::AttackArea(m) | GameMove::AttackCity(m) => {
            let takes_city = matches!(mv, GameMove::AttackCity(_));
            // Rule 24.31: an attack on a city that can't reach 7 (8) tokens this
            // turn just loses the attackers. Veto it like any losing fight.
            if takes_city && !rivals.sieges.is_target(m.target) {
                return 0.02 * w.aggression;
            }
            let base = score_attack(m.source, m.target, m.max_tokens, takes_city, areas, w);
            // The conquest bonus goes on *after* the clamp: good attacks already
            // saturate the 1.5 cap, which would hide "hit the leader, not the
            // straggler" entirely.
            clamp01_soft(base) + conquest_bonus(m.target, m.max_tokens, takes_city, areas, rivals)
        }
        // Not a movement-phase move; never selected here.
        _ => f32::NEG_INFINITY,
    }
}

/// Tokens above an area's limit will be removed at end of turn (surplus
/// removal) unless a city is built there, so moving them out is free. Without
/// this, a cautious AI sat on 2 tokens in a capacity-1 area for rounds on end,
/// losing the extra one every turn instead of spreading it.
fn surplus_bonus(source: Entity, areas: &HashMap<Entity, AreaSummary>, plan: &CityPlan) -> f32 {
    if plan.is_target(source) || plan.spare_at(source).is_some() {
        return 0.0; // those extra tokens are for the city
    }
    match area(areas, source) {
        Some(src) if src.my_pop > src.max_population => 0.3,
        _ => 0.0,
    }
}

/// Rival standings for a Conqueror-style "attack the leader" pressure.
#[derive(Clone, Debug, Default)]
pub struct Rivals {
    /// Strategy's appetite for this (0 = ignore standings entirely).
    pub conquest: f32,
    /// Rival -> how far ahead of us they are (0..1, 0.5 = level); see
    /// `rival_standing`.
    pub standing: HashMap<Entity, f32>,
    /// Enemy cities that can actually be taken this turn (rule 24.31). City
    /// attacks outside this plan are vetoed.
    pub sieges: SiegePlan,
}

/// Extra value for a *winnable* attack on a rival who's ahead: more for taking
/// a city (+50 for us, -50 for them, possibly freezing their A.S.T.).
fn conquest_bonus(
    target: Entity,
    max_tokens: usize,
    takes_city: bool,
    areas: &HashMap<Entity, AreaSummary>,
    rivals: &Rivals,
) -> f32 {
    if rivals.conquest <= 0.0 {
        return 0.0;
    }
    let Some(tgt) = area(areas, target) else {
        return 0.0;
    };
    let attackers = max_tokens.saturating_sub(1).max(1);
    if attackers <= tgt.enemy_pop.max(1) {
        return 0.0; // never sweeten a losing or even fight
    }
    let owner = if takes_city {
        tgt.city_owner
    } else {
        tgt.main_enemy
    };
    let standing = owner
        .and_then(|o| rivals.standing.get(&o).copied())
        .unwrap_or(0.5);
    let scale = if takes_city { 1.0 } else { 0.5 };
    rivals.conquest * (standing - 0.3).max(0.0) * scale * 1.4
}

/// Bonus for feeding a planned city site, penalty for draining one. The bonus
/// grows as the site nears the threshold so tokens converge on one site instead
/// of trickling into several.
fn city_plan_adjustment(
    source: Entity,
    target: Entity,
    areas: &HashMap<Entity, AreaSummary>,
    plan: &CityPlan,
) -> f32 {
    if plan.drive <= 0.0 {
        return 0.0;
    }
    if plan.is_target(source) || plan.spare_at(source) == Some(0) {
        return -plan.drive;
    }
    if plan.is_target(target) {
        let my_pop = area(areas, target).map_or(0, |a| a.my_pop) as f32;
        let progress = (my_pop / plan.threshold.max(1) as f32).min(1.0);
        return plan.drive * (0.8 + 0.5 * progress);
    }
    0.0
}

fn area(areas: &HashMap<Entity, AreaSummary>, e: Entity) -> Option<&AreaSummary> {
    areas.get(&e)
}

/// The "do nothing / hold position" baseline. Its value is mostly the *defensive*
/// worth of staying put: holding city squares and not exposing positions. A modest
/// floor so the AI doesn't shuffle tokens around for no gain.
fn score_end_movement(player: Entity, areas: &HashMap<Entity, AreaSummary>, w: &Weights) -> f32 {
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
        let thinness = if remaining == 0 {
            1.0
        } else {
            1.0 / remaining as f32
        };
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
    use crate::stupid_ai::{Personality, Playstyle, Strategy};

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
                my_pop: 9,
                enemy_pop: 0,
                neighbours: vec![enemy],
                ..Default::default()
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
                ..Default::default()
            },
        );
        (player, home, enemy, areas)
    }

    #[test]
    fn warlord_prefers_taking_a_winnable_city_over_holding() {
        let (player, home, enemy, areas) = winnable_city_world();
        let w = Personality::from_playstyle(Playstyle::Warlord).weights;

        let attack = GameMove::AttackCity(MovementMove::new(home, enemy, player, 9));
        let hold = GameMove::EndMovement;
        let rivals = Rivals {
            sieges: SiegePlan::build(&HashMap::from_iter([(1, attack.clone())]), &areas),
            ..Default::default()
        };
        assert!(rivals.sieges.is_target(enemy));

        let attack_score = score_movement(
            &attack,
            player,
            &areas,
            &w,
            &CityPlan::default(),
            &rivals,
        );
        let hold_score = score_movement(
            &hold,
            player,
            &areas,
            &w,
            &CityPlan::default(),
            &rivals,
        );
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
        let attack_score = score_movement(
            &attack,
            player,
            &areas,
            &w,
            &CityPlan::default(),
            &Rivals::default(),
        );
        let hold_score = score_movement(
            &GameMove::EndMovement,
            player,
            &areas,
            &w,
            &CityPlan::default(),
            &Rivals::default(),
        );
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
            let ferry_score = score_movement(
                &empty_ferry,
                player,
                &areas,
                &w,
                &CityPlan::default(),
                &Rivals::default(),
            );
            let hold_score = score_movement(
                &GameMove::EndMovement,
                player,
                &areas,
                &w,
                &CityPlan::default(),
                &Rivals::default(),
            );
            assert!(
                ferry_score <= hold_score,
                "{playstyle:?}: empty ferry ({ferry_score}) should never outscore EndMovement ({hold_score})"
            );
        }
    }

    /// The zero-city bug: a player whose site is 3 tokens short, with 3 movable
    /// tokens next door, used to prefer spreading into empty land every time, so
    /// no stack ever reached 6. With a city plan (drive from AST urgency), every
    /// playstyle must prefer feeding the site — and must not drain it.
    #[test]
    fn city_plan_beats_grabbing_land_for_every_playstyle() {
        let player = e(1);
        let site = e(10);
        let feeder = e(20);
        let empty = e(30);
        let mut areas = HashMap::default();
        areas.insert(
            site,
            AreaSummary {
                max_population: 3,
                is_city_site: true,
                my_pop: 3,
                neighbours: vec![feeder],
                ..Default::default()
            },
        );
        areas.insert(
            feeder,
            AreaSummary {
                max_population: 3,
                my_pop: 3,
                neighbours: vec![site, empty],
                ..Default::default()
            },
        );
        areas.insert(
            empty,
            AreaSummary {
                max_population: 4,
                is_city_site: true,
                neighbours: vec![feeder],
                ..Default::default()
            },
        );
        let feed = GameMove::Movement(MovementMove::new(feeder, site, player, 3));
        let grab = GameMove::Movement(MovementMove::new(feeder, empty, player, 3));
        let drain = GameMove::Movement(MovementMove::new(site, feeder, player, 3));
        let moves = HashMap::from_iter([(1, feed.clone()), (2, grab.clone()), (3, drain.clone())]);

        for playstyle in Playstyle::ALL {
            let w = Personality::from_playstyle(playstyle).weights;
            let plan = CityPlan::build(&moves, &areas, 6, w.city_income.max(1.0));
            assert_eq!(plan.needed_at(site), Some(3), "{playstyle:?}");
            let feed_s = score_movement(&feed, player, &areas, &w, &plan, &Rivals::default());
            let grab_s = score_movement(&grab, player, &areas, &w, &plan, &Rivals::default());
            let drain_s = score_movement(&drain, player, &areas, &w, &plan, &Rivals::default());
            let hold_s = score_movement(
                &GameMove::EndMovement,
                player,
                &areas,
                &w,
                &plan,
                &Rivals::default(),
            );
            assert!(
                feed_s > grab_s && feed_s > hold_s,
                "{playstyle:?}: feed {feed_s} should beat grab {grab_s} and hold {hold_s}"
            );
            assert!(
                drain_s < hold_s,
                "{playstyle:?}: drain {drain_s} vs hold {hold_s}"
            );
        }
    }

    /// Iberia-as-Turtle spent 12 rounds piling its whole stack into one full
    /// area each turn and losing the overflow to surplus removal. Moving into a
    /// full, unplanned area must rank below simply ending movement.
    #[test]
    fn moving_into_a_full_area_never_beats_ending_movement() {
        let player = e(1);
        let src = e(10);
        let full = e(20);
        let mut areas = HashMap::default();
        areas.insert(
            src,
            AreaSummary {
                max_population: 2,
                is_city_site: true,
                my_pop: 4,
                neighbours: vec![full],
                ..Default::default()
            },
        );
        areas.insert(
            full,
            AreaSummary {
                max_population: 2,
                is_city_site: true,
                my_pop: 2,
                neighbours: vec![src],
                ..Default::default()
            },
        );
        let pile = GameMove::Movement(MovementMove::new(src, full, player, 2));
        for playstyle in Playstyle::ALL {
            let w = Personality::from_playstyle(playstyle).weights;
            let plan = CityPlan::default();
            let pile_s = score_movement(&pile, player, &areas, &w, &plan, &Rivals::default());
            let hold_s = score_movement(
                &GameMove::EndMovement,
                player,
                &areas,
                &w,
                &plan,
                &Rivals::default(),
            );
            assert!(
                pile_s < hold_s,
                "{playstyle:?}: pile {pile_s} vs hold {hold_s}"
            );
        }
    }

    /// Conqueror: two equally winnable enemy cities; the one owned by the
    /// leader must score higher than the straggler's. And no strategy bonus
    /// may turn an even fight into an attractive one.
    #[test]
    fn conqueror_aims_at_the_leader_but_not_at_bad_odds() {
        let player = e(1);
        let leader = e(2);
        let straggler = e(3);
        let home = e(10);
        let leader_city = e(20);
        let straggler_city = e(30);
        let city = |owner: Entity, enemy_pop: usize| AreaSummary {
            max_population: 3,
            is_city_site: true,
            has_city: true,
            enemy_pop,
            neighbours: vec![home],
            city_owner: Some(owner),
            main_enemy: Some(owner),
            ..Default::default()
        };
        let mut areas = HashMap::default();
        areas.insert(
            home,
            AreaSummary {
                max_population: 3,
                my_pop: 10,
                neighbours: vec![leader_city, straggler_city],
                ..Default::default()
            },
        );
        areas.insert(leader_city, city(leader, 1));
        areas.insert(straggler_city, city(straggler, 1));
        let rivals = Rivals {
            conquest: Strategy::Conqueror.conquest(),
            standing: HashMap::from_iter([(leader, 0.95), (straggler, 0.1)]),
            sieges: SiegePlan::default()
                .with_target(leader_city, 8)
                .with_target(straggler_city, 8),
        };
        let w = Personality::new(Playstyle::Balanced, Strategy::Conqueror).weights;
        let plan = CityPlan::default();
        let hit_leader = GameMove::AttackCity(MovementMove::new(home, leader_city, player, 10));
        let hit_straggler =
            GameMove::AttackCity(MovementMove::new(home, straggler_city, player, 10));
        let l = score_movement(&hit_leader, player, &areas, &w, &plan, &rivals);
        let s = score_movement(&hit_straggler, player, &areas, &w, &plan, &rivals);
        assert!(l > s, "leader {l} should beat straggler {s}");

        // Even fight (1 attacker after keeping one back vs 1 defender): no
        // conquest bonus on top of the siege.
        let even = GameMove::AttackCity(MovementMove::new(home, leader_city, player, 2));
        let no_conquest = Rivals {
            sieges: rivals.sieges.clone(),
            ..Default::default()
        };
        let with = score_movement(&even, player, &areas, &w, &plan, &rivals);
        let without = score_movement(&even, player, &areas, &w, &plan, &no_conquest);
        assert!((with - without).abs() < 1e-6, "{with} vs {without}");

        // And with no siege plan at all, a city attack is vetoed outright.
        let unplanned = score_movement(&hit_leader, player, &areas, &w, &plan, &Rivals::default());
        let hold = score_movement(&GameMove::EndMovement, player, &areas, &w, &plan, &Rivals::default());
        assert!(unplanned < hold, "unplanned {unplanned} vs hold {hold}");
    }

    /// Surplus tokens are free: a Turtle with 2 tokens in a capacity-1 area
    /// should move the extra one to empty land rather than lose it.
    #[test]
    fn turtle_moves_surplus_out_instead_of_losing_it() {
        let player = e(1);
        let cramped = e(10);
        let empty = e(20);
        let mut areas = HashMap::default();
        areas.insert(
            cramped,
            AreaSummary {
                max_population: 1,
                my_pop: 2,
                neighbours: vec![empty],
                ..Default::default()
            },
        );
        areas.insert(
            empty,
            AreaSummary {
                max_population: 1,
                neighbours: vec![cramped],
                ..Default::default()
            },
        );
        let w = Personality::from_playstyle(Playstyle::Turtle).weights;
        let out = GameMove::Movement(MovementMove::new(cramped, empty, player, 2));
        let plan = CityPlan::default();
        let r = Rivals::default();
        let move_s = score_movement(&out, player, &areas, &w, &plan, &r);
        let hold_s = score_movement(&GameMove::EndMovement, player, &areas, &w, &plan, &r);
        assert!(move_s > hold_s + 0.2, "move {move_s} vs hold {hold_s}");
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
        let grab_score = score_movement(
            &grab,
            player,
            &areas,
            &w,
            &CityPlan::default(),
            &Rivals::default(),
        );
        let hold_score = score_movement(
            &GameMove::EndMovement,
            player,
            &areas,
            &w,
            &CityPlan::default(),
            &Rivals::default(),
        );
        assert!(
            grab_score > hold_score,
            "expansionist should move into the empty city site: grab {grab_score} vs hold {hold_score}"
        );
    }
}
