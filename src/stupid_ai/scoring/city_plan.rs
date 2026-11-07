//! Movement-phase city planning: which city sites can this player fill up to the
//! build threshold *this turn*?
//!
//! Why this exists: no area on the board has `max_population` >= 6, so Population
//! Expansion can never produce a buildable stack on its own — a city only ever
//! happens because movement *concentrates* tokens above an area's printed limit
//! (rule 24.1/24.2: 6 tokens on a city site, 12 elsewhere). The per-move scorer
//! is myopic and naturally spreads tokens out (grabbing new land always beat
//! reinforcing), so in self-play some factions held 20+ areas yet never stacked
//! more than 4 tokens anywhere and so never got a single build move offered. The
//! plan gives the scorer a concrete, reachable target to pile into.

use super::AreaSummary;
use crate::civilization::GameMove;
use bevy::platform::collections::HashMap;
use bevy::prelude::Entity;

#[derive(Clone, Debug, Default)]
pub struct CityPlan {
    /// Tokens needed on a city site to build (6, or 5 with Architecture).
    pub threshold: usize,
    /// How hard to pursue the plan, `max(city_income, urgency)`. 0 = no plan.
    pub drive: f32,
    /// Planned target area -> tokens still missing there.
    targets: HashMap<Entity, usize>,
}

impl CityPlan {
    /// Build the plan from the player's currently legal movement moves.
    ///
    /// A move's `max_tokens` is the number of not-yet-moved tokens in its source,
    /// so it doubles as that source's remaining supply. Candidates are empty-of-
    /// enemies, city-less city sites where `my_pop + reachable supply >= threshold`.
    /// They are claimed closest-to-done first, and each source's supply is spent
    /// on at most one target, so two half-filled sites don't split the tokens
    /// and both fail.
    pub fn build(
        moves: &HashMap<usize, GameMove>,
        areas: &HashMap<Entity, AreaSummary>,
        threshold: usize,
        drive: f32,
    ) -> Self {
        let mut plan = CityPlan {
            threshold,
            drive,
            targets: HashMap::default(),
        };
        if drive <= 0.0 || threshold == 0 {
            return plan;
        }

        // target -> [(source, supply)]
        let mut feeders: HashMap<Entity, Vec<(Entity, usize)>> = HashMap::default();
        let mut supply: HashMap<Entity, usize> = HashMap::default();
        for mv in moves.values() {
            let (GameMove::Movement(m) | GameMove::ShipFerry(m)) = mv else {
                continue;
            };
            if m.max_tokens == 0 || m.source == m.target {
                continue;
            }
            feeders
                .entry(m.target)
                .or_default()
                .push((m.source, m.max_tokens));
            let s = supply.entry(m.source).or_insert(0);
            *s = (*s).max(m.max_tokens);
        }

        let mut candidates: Vec<(Entity, usize, usize)> = feeders
            .keys()
            .filter_map(|t| {
                let a = areas.get(t)?;
                if !a.is_city_site || a.has_city || a.enemy_pop > 0 || a.my_pop >= threshold {
                    return None;
                }
                Some((*t, threshold - a.my_pop, a.max_population))
            })
            .collect();
        // Closest to done first; prefer bigger sites (they also hold more after),
        // then entity order for determinism.
        candidates.sort_by_key(|(t, needed, cap)| (*needed, std::cmp::Reverse(*cap), *t));

        for (target, needed, _) in candidates {
            // Tokens sitting on another planned target are already committed.
            let mut sources: Vec<(Entity, usize)> = feeders[&target]
                .iter()
                .filter(|(s, _)| !plan.targets.contains_key(s))
                .map(|(s, _)| (*s, supply.get(s).copied().unwrap_or(0)))
                .filter(|(_, n)| *n > 0)
                .collect();
            sources.sort_by_key(|(s, n)| (std::cmp::Reverse(*n), *s));
            sources.dedup_by_key(|(s, _)| *s);
            let available: usize = sources.iter().map(|(_, n)| n).sum();
            if available < needed {
                continue;
            }
            let mut remaining = needed;
            for (s, n) in sources {
                if remaining == 0 {
                    break;
                }
                let take = n.min(remaining);
                remaining -= take;
                if let Some(left) = supply.get_mut(&s) {
                    *left -= take;
                }
            }
            plan.targets.insert(target, needed);
        }
        plan
    }

    /// Tokens still needed at `area`, if it's a planned build target.
    pub fn needed_at(&self, area: Entity) -> Option<usize> {
        self.targets.get(&area).copied()
    }

    pub fn is_target(&self, area: Entity) -> bool {
        self.targets.contains_key(&area)
    }

    pub fn has_targets(&self) -> bool {
        !self.targets.is_empty()
    }

    /// How many tokens to send along a peaceful move into a planned target:
    /// exactly what's missing, capped by what can move.
    pub fn tokens_to_send(&self, target: Entity, max_tokens: usize) -> Option<usize> {
        self.needed_at(target).map(|n| n.min(max_tokens).max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::MovementMove;

    fn e(i: u32) -> Entity {
        Entity::from_raw_u32(i).unwrap()
    }

    fn site(my_pop: usize) -> AreaSummary {
        AreaSummary {
            max_population: 3,
            is_city_site: true,
            my_pop,
            ..Default::default()
        }
    }

    fn plain(my_pop: usize) -> AreaSummary {
        AreaSummary {
            max_population: 3,
            my_pop,
            ..Default::default()
        }
    }

    fn mv(src: Entity, tgt: Entity, n: usize) -> GameMove {
        GameMove::Movement(MovementMove::new(src, tgt, e(1), n))
    }

    #[test]
    fn reachable_site_becomes_a_target() {
        let (s, a, b) = (e(10), e(20), e(30));
        let areas = HashMap::from_iter([(s, site(3)), (a, plain(2)), (b, plain(2))]);
        let moves = HashMap::from_iter([(1, mv(a, s, 2)), (2, mv(b, s, 2))]);
        let plan = CityPlan::build(&moves, &areas, 6, 1.0);
        assert_eq!(plan.needed_at(s), Some(3));
    }

    #[test]
    fn unreachable_site_is_not_a_target() {
        let (s, a) = (e(10), e(20));
        let areas = HashMap::from_iter([(s, site(2)), (a, plain(3))]);
        let moves = HashMap::from_iter([(1, mv(a, s, 3))]);
        let plan = CityPlan::build(&moves, &areas, 6, 1.0);
        assert!(!plan.has_targets());
    }

    #[test]
    fn architecture_threshold_makes_it_reachable() {
        let (s, a) = (e(10), e(20));
        let areas = HashMap::from_iter([(s, site(2)), (a, plain(3))]);
        let moves = HashMap::from_iter([(1, mv(a, s, 3))]);
        let plan = CityPlan::build(&moves, &areas, 5, 1.0);
        assert_eq!(plan.needed_at(s), Some(3));
    }

    #[test]
    fn enemy_occupied_or_built_sites_are_skipped() {
        let (s1, s2, a) = (e(10), e(11), e(20));
        let mut contested = site(4);
        contested.enemy_pop = 1;
        let mut built = site(4);
        built.has_city = true;
        let areas = HashMap::from_iter([(s1, contested), (s2, built), (a, plain(3))]);
        let moves = HashMap::from_iter([(1, mv(a, s1, 3)), (2, mv(a, s2, 3))]);
        assert!(!CityPlan::build(&moves, &areas, 6, 1.0).has_targets());
    }

    /// One 3-token source next to two sites at 3 each: only one can be fed, and
    /// the shared supply must not be double-counted for both.
    #[test]
    fn shared_supply_is_spent_once() {
        let (s1, s2, a) = (e(10), e(11), e(20));
        let areas = HashMap::from_iter([(s1, site(3)), (s2, site(3)), (a, plain(3))]);
        let moves = HashMap::from_iter([(1, mv(a, s1, 3)), (2, mv(a, s2, 3))]);
        let plan = CityPlan::build(&moves, &areas, 6, 1.0);
        assert_eq!(
            [s1, s2].iter().filter(|t| plan.is_target(**t)).count(),
            1,
            "exactly one site should be planned"
        );
    }

    #[test]
    fn zero_drive_means_no_plan() {
        let (s, a) = (e(10), e(20));
        let areas = HashMap::from_iter([(s, site(5)), (a, plain(3))]);
        let moves = HashMap::from_iter([(1, mv(a, s, 3))]);
        assert!(!CityPlan::build(&moves, &areas, 6, 0.0).has_targets());
    }

    #[test]
    fn tokens_to_send_is_exactly_whats_missing() {
        let (s, a) = (e(10), e(20));
        let areas = HashMap::from_iter([(s, site(5)), (a, plain(3))]);
        let moves = HashMap::from_iter([(1, mv(a, s, 3))]);
        let plan = CityPlan::build(&moves, &areas, 6, 1.0);
        assert_eq!(plan.tokens_to_send(s, 3), Some(1));
        assert_eq!(plan.tokens_to_send(e(99), 3), None);
    }
}
