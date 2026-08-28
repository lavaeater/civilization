use super::{AreaSummary, enemy_pressure, saturating};
use crate::civilization::GameMove;
use crate::stupid_ai::Weights;
use bevy::platform::collections::HashMap;
use bevy::prelude::Entity;

/// Score a city-construction-phase move. A city is the game's income/win engine, so
/// building is almost always good; personality decides *how* eager and whether the
/// site is worth committing tokens to.
///
/// `urgency` (0..=1) overrides personality when a player is short of the cities
/// needed to advance past their next A.S.T. epoch gate (rule 33.11/33.3): no
/// playstyle is allowed to sit at zero cities forever just because `city_income`
/// is a low-priority knob for it — see the M6 tuning note in
/// `docs/utility-ai-design.md`.
pub fn score_city_construction(
    mv: &GameMove,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
    urgency: f32,
) -> f32 {
    let effective_city_income = w.city_income.max(urgency);
    match mv {
        GameMove::CityConstruction(m) => {
            let Some(tgt) = areas.get(&m.target) else {
                return 0.0;
            };
            let capacity = saturating(tgt.max_population as f32, 4.0);
            let mut score = effective_city_income * (0.6 + 0.4 * capacity);
            // A defensible site (little enemy pressure) is worth more; an exposed
            // one risks being attrited/captured.
            let pressure = enemy_pressure(m.target, areas) as f32;
            score += w.defense * (0.2 - 0.2 * saturating(pressure, 4.0));
            score
        }
        // The "build nothing more" baseline: attractive to players who'd rather keep
        // tokens mobile (low city_income, some aggression) than commit them to a city.
        // Urgency also damps the aggression term — a player stuck at zero cities
        // shouldn't be tempted to skip building just because they're a Warlord.
        GameMove::EndCityConstruction => {
            0.3 * (1.0 - effective_city_income) + 0.1 * w.aggression * (1.0 - urgency)
        }
        _ => f32::NEG_INFINITY,
    }
}

/// Score a forced city-elimination choice. The game makes us give up a city; a
/// *higher* score means "this is the better one to lose". So we prefer to shed
/// exposed, low-value cities and keep defensible, high-capacity ones.
pub fn score_city_elimination(
    mv: &GameMove,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    let GameMove::EliminateCity(m) = mv else {
        return f32::NEG_INFINITY;
    };
    let Some(area) = areas.get(&m.area) else {
        return 0.0;
    };
    let exposure = saturating(enemy_pressure(m.area, areas) as f32, 4.0);
    let board_value = saturating(area.max_population as f32, 4.0);
    // Drop the exposed one (defense likes shedding liabilities), keep the valuable
    // one (city_income hates losing capacity).
    w.defense * exposure - w.city_income * board_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::BuildCityMove;
    use crate::stupid_ai::{Personality, Playstyle};

    fn e(i: u32) -> Entity {
        Entity::from_raw_u32(i).unwrap()
    }

    fn one_area_world(max_population: usize) -> (Entity, HashMap<Entity, AreaSummary>) {
        let target = e(10);
        let mut areas = HashMap::default();
        areas.insert(
            target,
            AreaSummary {
                max_population,
                my_pop: max_population,
                ..Default::default()
            },
        );
        (target, areas)
    }

    #[test]
    fn balanced_player_prefers_building_with_no_urgency() {
        let (target, areas) = one_area_world(4);
        let w = Personality::from_playstyle(Playstyle::Balanced).weights;
        let build = GameMove::CityConstruction(BuildCityMove::new(target, e(1)));
        let end = GameMove::EndCityConstruction;

        let build_score = score_city_construction(&build, &areas, &w, 0.0);
        let end_score = score_city_construction(&end, &areas, &w, 0.0);
        assert!(
            build_score > end_score,
            "build {build_score} vs end {end_score}"
        );
    }

    /// The bug this test guards against: a Warlord has low `city_income`, so
    /// without an urgency override it can rank `EndCityConstruction` above
    /// building even a perfectly good city — leaving it permanently frozen on
    /// the A.S.T. for lack of cities (rule 33.11/33.3).
    #[test]
    fn urgency_forces_warlord_to_build_even_though_city_income_is_low() {
        let (target, areas) = one_area_world(4);
        let w = Personality::from_playstyle(Playstyle::Warlord).weights;
        let build = GameMove::CityConstruction(BuildCityMove::new(target, e(1)));
        let end = GameMove::EndCityConstruction;

        // Without urgency a Warlord may still prefer building here, but the
        // margin is thin; with urgency it must not lose.
        let build_score = score_city_construction(&build, &areas, &w, 1.0);
        let end_score = score_city_construction(&end, &areas, &w, 1.0);
        assert!(
            build_score > end_score,
            "urgent build {build_score} vs end {end_score}"
        );
    }

    #[test]
    fn urgency_does_not_override_site_quality() {
        // A very poor site (tiny capacity, heavy enemy pressure) should still
        // score below one it's cheap to defend, even under urgency — urgency
        // says "build something", not "build anywhere blindly".
        let good_target = e(10);
        let bad_target = e(20);
        let enemy_neighbour = e(30);
        let mut areas = HashMap::default();
        areas.insert(
            good_target,
            AreaSummary {
                max_population: 8,
                my_pop: 8,
                ..Default::default()
            },
        );
        areas.insert(
            bad_target,
            AreaSummary {
                max_population: 1,
                my_pop: 1,
                neighbours: vec![enemy_neighbour],
                ..Default::default()
            },
        );
        areas.insert(
            enemy_neighbour,
            AreaSummary {
                enemy_pop: 4,
                ..Default::default()
            },
        );
        let w = Personality::from_playstyle(Playstyle::Turtle).weights;
        let good = GameMove::CityConstruction(BuildCityMove::new(good_target, e(1)));
        let bad = GameMove::CityConstruction(BuildCityMove::new(bad_target, e(1)));

        let good_score = score_city_construction(&good, &areas, &w, 1.0);
        let bad_score = score_city_construction(&bad, &areas, &w, 1.0);
        assert!(
            good_score > bad_score,
            "good site {good_score} vs bad site {bad_score}"
        );
    }
}
