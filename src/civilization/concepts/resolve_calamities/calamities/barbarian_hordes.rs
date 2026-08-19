use bevy::prelude::{Entity, Reflect};

/// State for Barbarian Hordes (rule 30.52).
///
/// Barbarians are represented as a lightweight, ephemeral non-`Player` owner
/// entity (`barbarian_entity`, spawned in `PlaceBarbarians`) holding real
/// `Token` entities in `Population`, so they participate in this game's
/// existing conflict-resolution machinery (`UnresolvedConflict`/
/// `UnresolvedCityConflict` + the observers in `conflict_triggers.rs`)
/// exactly like any other faction's tokens would -- no bespoke combat math.
/// The owner entity has none of `Player`/`Treasury`/`PlayerTradeCards`/
/// `PlayerCities`/`TokenStock`/`PlayerAreas`, which is what naturally makes
/// it immune to trading, taxation, city-support, etc.
///
/// KNOWN SIMPLIFICATION (documented, not silently dropped -- see
/// docs/outline.md): rule 30.5235 says surviving Barbarians remain on the
/// board until eliminated by a later conflict. This implementation instead
/// despawns all Barbarian tokens (winners included) at `Complete`. Making
/// them a persistent nation that correctly interacts with every subsequent
/// phase for the rest of the game (census, movement, further conflicts,
/// city construction, save/load, ...) is a materially larger integration
/// project than this pass; real placement + real conflict resolution + a
/// real movement cascade is still a large upgrade over the previous flat
/// unit-point approximation.
#[derive(Debug, Clone, Default, Reflect)]
pub struct BarbarianHordesState {
    pub phase: BarbarianHordesPhase,
    /// The area Barbarians currently occupy (landing area, or wherever the
    /// cascade has advanced to).
    pub landing_area: Option<Entity>,
    /// The lightweight non-`Player` entity that owns Barbarian tokens in
    /// `Population`. `None` until `PlaceBarbarians` spawns it.
    pub barbarian_entity: Option<Entity>,
    /// Every area Barbarians have occupied at any point this resolution --
    /// needed at `Complete` to sweep up survivors left behind at each stop
    /// along the cascade (30.5231: only the *surplus* moves on; the rest
    /// stays), not just the final area.
    pub visited_areas: Vec<Entity>,
    /// Every token entity ever spawned for this Barbarian nation (always 15,
    /// spawned once in `PlaceBarbarians` -- the cascade only moves existing
    /// tokens between areas' `Population`, it never creates more). Tracked
    /// separately from `visited_areas` so `Complete` can despawn every one
    /// of them without leaking an entity, whether it's a survivor still
    /// listed in some area's `Population` or a token already eliminated in
    /// an earlier conflict (which -- lacking a `TokenStock` to return to --
    /// would otherwise be left as a harmless but permanent orphan entity).
    pub all_tokens: Vec<Entity>,
    /// Counts hops in the movement cascade, capped at `MAX_CASCADE_ITERATIONS`.
    pub cascade_iterations: usize,
}

/// Safety bound on the movement cascade (30.5231/30.5232 repeats "until no
/// surplus Barbarian tokens remain", which is not rules-guaranteed to
/// terminate quickly on a pathological board). This is a defensive
/// engineering bound, not a rule -- 20 hops is far more than any real
/// board's diameter.
pub const MAX_CASCADE_ITERATIONS: usize = 20;

// Note: a prior version of this module modeled Barbarian Hordes as a flat
// "-5 unit points with Military" approximation. Rule 30.52 never mentions
// Military at all (same class of fabricated-modifier bug as Civil War's
// wrong Philosophy/Military mechanics, fixed earlier this session) -- that
// field and method have been removed along with the flat-loss mechanic.

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum BarbarianHordesPhase {
    #[default]
    FindLandingArea,
    /// Spawn the Barbarian owner entity and its 15 tokens in `landing_area`.
    /// Initial-placement only; the cascade moves existing tokens instead.
    PlaceBarbarians,
    /// Trigger real conflict resolution on `landing_area` if it qualifies as
    /// a conflict zone (population.is_conflict_zone); otherwise skip
    /// straight to `CheckSurplus` -- landing in a genuinely empty area with
    /// no city is not a fight.
    EnterArea,
    /// Waiting for `UnresolvedConflict`/`UnresolvedCityConflict` to clear
    /// from `landing_area` (the conflict observers resolve synchronously
    /// once Commands flush, which happens after this system returns -- so
    /// this is a real poll across frames, matching this codebase's
    /// established "wait and re-check next tick" pattern).
    AwaitingConflict,
    /// Compute surplus (30.5231/30.5232) in `landing_area`; if none (or the
    /// cascade cap is reached, or there's nowhere left to go), advance to
    /// `Complete`. Otherwise move the surplus to the adjacent area causing
    /// the greatest damage (30.5234) and loop back to `EnterArea`.
    CheckSurplus,
    Complete,
}

impl BarbarianHordesState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Rule 30.5211/30.5234: "damage to the primary victim" in a candidate area.
/// Tokens count 1 point each, an owned city counts 5 -- the same unit-point
/// convention this codebase already uses elsewhere (e.g. taxation's revolt
/// beneficiary selection: tokens=1, cities=5), used here for both initial
/// placement and every cascade step, matching the rule's identical "greatest
/// damage" wording in both 30.5211 and 30.5234.
pub fn barbarian_damage_score(victim_tokens: usize, victim_has_city: bool) -> usize {
    victim_tokens + if victim_has_city { 5 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_starts_at_find_landing_area_with_no_area_or_owner_yet() {
        let state = BarbarianHordesState::new();
        assert_eq!(state.phase, BarbarianHordesPhase::FindLandingArea);
        assert!(state.landing_area.is_none());
        assert!(state.barbarian_entity.is_none());
        assert!(state.visited_areas.is_empty());
        assert!(state.all_tokens.is_empty());
        assert_eq!(state.cascade_iterations, 0);
    }

    // ── Rule 30.5211/30.5234: greatest-damage scoring ───────────────────────

    #[test]
    fn damage_score_counts_tokens_at_1_point_each() {
        assert_eq!(barbarian_damage_score(7, false), 7);
    }

    #[test]
    fn damage_score_adds_5_for_an_owned_city() {
        assert_eq!(barbarian_damage_score(3, true), 8);
    }

    #[test]
    fn an_empty_area_with_no_city_scores_zero_damage() {
        assert_eq!(barbarian_damage_score(0, false), 0);
    }

    #[test]
    fn a_bare_city_with_no_defenders_still_scores_as_damage() {
        // A city with 0 tokens in Population (all garrison lost) is still a
        // valid, damaging target -- matches 24.x's city=5-points convention.
        assert_eq!(barbarian_damage_score(0, true), 5);
    }
}
