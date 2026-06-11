use super::{enemy_pressure, saturating, AreaSummary};
use crate::civilization::GameMove;
use crate::stupid_ai::Weights;
use bevy::platform::collections::HashMap;
use bevy::prelude::Entity;

/// Score a city-construction-phase move. A city is the game's income/win engine, so
/// building is almost always good; personality decides *how* eager and whether the
/// site is worth committing tokens to.
pub fn score_city_construction(
    mv: &GameMove,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    match mv {
        GameMove::CityConstruction(m) => {
            let Some(tgt) = areas.get(&m.target) else {
                return 0.0;
            };
            let capacity = saturating(tgt.max_population as f32, 4.0);
            let mut score = w.city_income * (0.6 + 0.4 * capacity);
            // A defensible site (little enemy pressure) is worth more; an exposed
            // one risks being attrited/captured.
            let pressure = enemy_pressure(m.target, areas) as f32;
            score += w.defense * (0.2 - 0.2 * saturating(pressure, 4.0));
            score
        }
        // The "build nothing more" baseline: attractive to players who'd rather keep
        // tokens mobile (low city_income, some aggression) than commit them to a city.
        GameMove::EndCityConstruction => 0.3 * (1.0 - w.city_income) + 0.1 * w.aggression,
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
