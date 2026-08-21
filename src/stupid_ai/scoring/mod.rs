//! Utility scoring over the enumerated `AvailableMoves` list. See
//! `docs/utility-ai-design.md`. Each `score_*` function is pure given the
//! pre-gathered [`AreaSummary`] map, so it can be unit-tested without an app.

mod city;
mod civ_cards;
mod expansion;
mod movement;
mod trade;
pub use city::*;
pub use civ_cards::*;
pub use expansion::*;
pub use movement::*;
pub use trade::*;

use crate::stupid_ai::{Personality, Picker};
use bevy::platform::collections::HashMap;
use bevy::prelude::Entity;
use rand::{Rng, RngExt};

/// A flattened, scoring-friendly snapshot of one area, gathered once per decision
/// so the scoring functions stay pure (no Bevy queries inside them).
#[derive(Clone, Debug, Default)]
pub struct AreaSummary {
    pub max_population: usize,
    pub is_city_site: bool,
    pub has_city: bool,
    pub city_is_mine: bool,
    /// Tokens the deciding player has in this area.
    pub my_pop: usize,
    /// Tokens belonging to *other* players in this area.
    pub enemy_pop: usize,
    pub neighbours: Vec<Entity>,
}

impl AreaSummary {
    /// Can this area host a city (and therefore generate tax/card income)?
    pub fn supports_city(&self) -> bool {
        self.is_city_site || self.max_population >= 6
    }

    pub fn is_empty(&self) -> bool {
        self.my_pop == 0 && self.enemy_pop == 0
    }
}

/// Total enemy tokens sitting in the areas adjacent to `area` — a proxy for how
/// much pressure a position is under.
pub fn enemy_pressure(area: Entity, areas: &HashMap<Entity, AreaSummary>) -> usize {
    let Some(summary) = areas.get(&area) else {
        return 0;
    };
    summary
        .neighbours
        .iter()
        .filter_map(|n| areas.get(n))
        .map(|n| n.enemy_pop)
        .sum()
}

// --- response curves (evaluators) ------------------------------------------

pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Diminishing-returns curve: rises fast then flattens. Good for "more is better
/// but the first units matter most" (e.g. attacker advantage, area capacity).
pub fn saturating(v: f32, half: f32) -> f32 {
    if v <= 0.0 { 0.0 } else { v / (v + half) }
}

// --- picking ----------------------------------------------------------------

/// Pick an index into `scored` according to the personality's [`Picker`].
/// `scored[i] = (move_index, score)`. Returns the chosen `move_index`.
pub fn pick<R: Rng>(scored: &[(usize, f32)], picker: Picker, rng: &mut R) -> Option<usize> {
    if scored.is_empty() {
        return None;
    }
    // Only finite scores are eligible. If a scorer hands us all-NaN or
    // all-`NEG_INFINITY` scores (e.g. a stale move set that doesn't match this
    // phase's scorer), every branch below would otherwise produce an empty
    // candidate set and panic in `random_range(0..0)`. Fall back to a uniform
    // pick over all moves so the game keeps moving instead of crashing.
    let finite: Vec<(usize, f32)> = scored
        .iter()
        .copied()
        .filter(|(_, s)| s.is_finite())
        .collect();
    if finite.is_empty() {
        return Some(scored[rng.random_range(0..scored.len())].0);
    }
    let scored = &finite[..];

    match picker {
        Picker::Greedy => {
            // Highest score; ties broken randomly so we don't always favour the
            // lowest entity id / insertion order.
            let max = scored
                .iter()
                .map(|(_, s)| *s)
                .fold(f32::NEG_INFINITY, f32::max);
            let leaders: Vec<usize> = scored
                .iter()
                .filter(|(_, s)| (*s - max).abs() < 1e-6)
                .map(|(i, _)| *i)
                .collect();
            // `leaders` is non-empty here: `scored` is non-empty and all finite,
            // so at least the max element matches. Guard anyway, defensively.
            if leaders.is_empty() {
                return Some(scored[0].0);
            }
            Some(leaders[rng.random_range(0..leaders.len())])
        }
        Picker::Softmax { temperature } => {
            let t = temperature.max(1e-3);
            let max = scored
                .iter()
                .map(|(_, s)| *s)
                .fold(f32::NEG_INFINITY, f32::max);
            let weights: Vec<f64> = scored
                .iter()
                .map(|(_, s)| f64::from((*s - max) / t).exp())
                .collect();
            let total: f64 = weights.iter().sum();
            if total <= 0.0 || !total.is_finite() {
                return Some(scored[rng.random_range(0..scored.len())].0);
            }
            let mut roll = rng.random_range(0.0..total);
            for (idx, w) in scored.iter().zip(weights.iter()) {
                roll -= *w;
                if roll <= 0.0 {
                    return Some(idx.0);
                }
            }
            Some(scored.last().unwrap().0)
        }
    }
}

/// Convenience: score every move with `score_fn`, then pick per personality.
pub fn score_and_pick<R, F>(
    moves: &HashMap<usize, crate::civilization::GameMove>,
    personality: &Personality,
    rng: &mut R,
    mut score_fn: F,
) -> Option<usize>
where
    R: Rng,
    F: FnMut(&crate::civilization::GameMove) -> f32,
{
    let scored: Vec<(usize, f32)> = moves.iter().map(|(i, m)| (*i, score_fn(m))).collect();
    pick(&scored, personality.picker, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A non-empty move set where every score is non-finite must still pick a
    /// move rather than panic in `random_range(0..0)`. This mirrors the real
    /// crash: the elimination scorer handed all `NEG_INFINITY` (wrong move type)
    /// and Greedy's tie filter `(NEG_INF - NEG_INF).abs() < eps` left no leaders.
    #[test]
    fn pick_does_not_panic_on_all_non_finite_scores() {
        let mut rng = rand::rng();
        let all_neg_inf = vec![(0usize, f32::NEG_INFINITY), (1, f32::NEG_INFINITY)];
        let all_nan = vec![(0usize, f32::NAN), (1, f32::NAN)];

        for picker in [Picker::Greedy, Picker::Softmax { temperature: 0.5 }] {
            for scored in [&all_neg_inf, &all_nan] {
                let chosen = pick(scored, picker, &mut rng);
                assert!(
                    chosen.is_some_and(|c| c == 0 || c == 1),
                    "pick should return one of the available moves, got {chosen:?}"
                );
            }
        }
    }

    /// When some scores are finite and some aren't, the finite ones win.
    #[test]
    fn pick_ignores_non_finite_scores_when_finite_exist() {
        let mut rng = rand::rng();
        let scored = vec![(0usize, f32::NEG_INFINITY), (1, 0.9), (2, f32::NAN)];
        // Greedy must land on the only finite (and highest) score, index 1.
        for _ in 0..20 {
            assert_eq!(pick(&scored, Picker::Greedy, &mut rng), Some(1));
        }
    }

    #[test]
    fn pick_returns_none_on_empty() {
        let mut rng = rand::rng();
        assert_eq!(pick(&[], Picker::Greedy, &mut rng), None);
    }
}
