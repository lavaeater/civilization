use super::{AreaSummary, saturating};
use crate::civilization::GameMove;
use crate::stupid_ai::Weights;
use bevy::platform::collections::HashMap;
use bevy::prelude::Entity;

/// Score where to place population growth during the (usually auto-resolved)
/// Population Expansion phase. Favour building up areas that can host a city, scaled
/// by the personality's growth/city focus.
pub fn score_population_expansion(
    mv: &GameMove,
    areas: &HashMap<Entity, AreaSummary>,
    w: &Weights,
) -> f32 {
    let GameMove::PopulationExpansion(m) = mv else {
        return f32::NEG_INFINITY;
    };
    let Some(area) = areas.get(&m.area) else {
        return w.growth * 0.5;
    };
    let capacity = saturating(area.max_population as f32, 4.0);
    let mut score = w.growth * 0.4;
    if area.supports_city() {
        score += w.city_income * (0.4 + 0.4 * capacity);
    }
    // Contested ground: expansionists/aggressors like pressing into it, cautious
    // players would rather not stack where a conflict looms.
    if area.enemy_pop > 0 {
        score += w.expansion * 0.2 - w.defense * 0.2;
    }
    score
}
