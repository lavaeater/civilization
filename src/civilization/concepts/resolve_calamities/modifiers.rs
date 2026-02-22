use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::resolve_calamities::context::{CalamityEffects, CalamityContext};
use crate::civilization::{CivCardName, TradeCard};

pub fn apply_calamity_modifiers(
    context: &CalamityContext,
    effects: &mut CalamityEffects,
    civ_cards: &PlayerCivilizationCards,
) {
    match context.calamity {
        TradeCard::VolcanoEarthquake => {
            apply_volcano_earthquake_modifiers(effects, civ_cards);
        }
        TradeCard::Famine => {
            apply_famine_modifiers(effects, civ_cards);
        }
        TradeCard::Superstition => {
            apply_superstition_modifiers(effects, civ_cards);
        }
        TradeCard::CivilWar => {
            apply_civil_war_modifiers(effects, civ_cards);
        }
        TradeCard::SlaveRevolt => {
            apply_slave_revolt_modifiers(effects, civ_cards);
        }
        TradeCard::Flood => {
            apply_flood_modifiers(effects, civ_cards);
        }
        TradeCard::BarbarianHordes => {
            apply_barbarian_hordes_modifiers(effects, civ_cards);
        }
        TradeCard::Epidemic => {
            apply_epidemic_modifiers(effects, civ_cards);
        }
        TradeCard::CivilDisorder => {
            apply_civil_disorder_modifiers(effects, civ_cards);
        }
        TradeCard::IconoclasmAndHeresy => {
            apply_iconoclasm_modifiers(effects, civ_cards);
        }
        TradeCard::Piracy => {
            apply_piracy_modifiers(effects, civ_cards);
        }
        TradeCard::Treachery => {
            // Treachery has no civ card modifiers
        }
        _ => {}
    }
}

fn apply_volcano_earthquake_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    if civ_cards.owns(&CivCardName::Engineering) {
        if effects.cities_to_destroy > 0 {
            effects.cities_to_destroy -= 1;
            effects.cities_to_reduce += 1;
        }
    }
}

fn apply_famine_modifiers(_effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Pottery: reduce losses by 4 unit points for each Grain card held
    // Note: Grain card count must be passed in separately, this is a placeholder
    if civ_cards.owns(&CivCardName::Pottery) {
        // Actual reduction depends on Grain cards held - handled at resolution time
    }
}

fn apply_superstition_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Mysticism: 2 cities reduced instead of 3
    // Deism: 1 city reduced
    // Enlightenment: no effect
    // Effects are NOT cumulative - highest level Religion card governs
    if civ_cards.owns(&CivCardName::Enlightenment) {
        effects.cities_to_reduce = 0;
    } else if civ_cards.owns(&CivCardName::Deism) {
        effects.cities_to_reduce = 1;
    } else if civ_cards.owns(&CivCardName::Mysticism) {
        effects.cities_to_reduce = 2;
    }
    // Default is 3 cities reduced (set by base effect)
}

fn apply_civil_war_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Music: +5 unit points to victim selection
    // Drama and Poetry: +5 unit points to victim selection
    // Democracy: +10 unit points to victim selection
    // Effects ARE cumulative
    if let Some(ref mut special) = effects.special_effect {
        if let crate::civilization::concepts::resolve_calamities::context::SpecialEffect::CivilWar(ref mut civil_war) = special {
            if civ_cards.owns(&CivCardName::Music) {
                civil_war.victim_selection_points += 5;
            }
            if civ_cards.owns(&CivCardName::DramaAndPoetry) {
                civil_war.victim_selection_points += 5;
            }
            if civ_cards.owns(&CivCardName::Democracy) {
                civil_war.victim_selection_points += 10;
            }
        }
    }
}

fn apply_slave_revolt_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Enlightenment: no effect
    if civ_cards.owns(&CivCardName::Enlightenment) {
        effects.cities_to_reduce = 0;
    }
}

fn apply_flood_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Engineering: protects against flood (city reduced instead of destroyed)
    if civ_cards.owns(&CivCardName::Engineering) {
        if effects.cities_to_destroy > 0 {
            effects.cities_to_destroy -= 1;
            effects.cities_to_reduce += 1;
        }
    }
}

fn apply_barbarian_hordes_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Military: reduces losses
    if civ_cards.owns(&CivCardName::Military) {
        effects.unit_points_to_lose = (effects.unit_points_to_lose - 5).max(0);
    }
}

fn apply_epidemic_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Medicine: reduces losses by half
    if civ_cards.owns(&CivCardName::Medicine) {
        effects.unit_points_to_lose /= 2;
    }
}

fn apply_civil_disorder_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Law: reduces cities reduced by 1
    // Democracy: no effect (immune)
    if civ_cards.owns(&CivCardName::Democracy) {
        effects.cities_to_reduce = 0;
    } else if civ_cards.owns(&CivCardName::Law) {
        effects.cities_to_reduce = effects.cities_to_reduce.saturating_sub(1);
    }
}

fn apply_iconoclasm_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // Theology: reduces losses
    // Philosophy: reduces losses
    if civ_cards.owns(&CivCardName::Theology) {
        effects.unit_points_to_lose = (effects.unit_points_to_lose - 4).max(0);
    }
    if civ_cards.owns(&CivCardName::Philosophy) {
        effects.unit_points_to_lose = (effects.unit_points_to_lose - 2).max(0);
    }
}

fn apply_piracy_modifiers(effects: &mut CalamityEffects, civ_cards: &PlayerCivilizationCards) {
    // No specific civ card modifiers for Piracy primary victim
    // Secondary victims can be protected by various cards
    let _ = civ_cards;
    let _ = effects;
}

pub fn can_be_secondary_victim(
    calamity: TradeCard,
    civ_cards: &PlayerCivilizationCards,
) -> bool {
    match calamity {
        TradeCard::VolcanoEarthquake => {
            // Engineering protects against being secondary victim of Earthquake
            !civ_cards.owns(&CivCardName::Engineering)
        }
        TradeCard::Epidemic => {
            // Medicine provides some protection
            !civ_cards.owns(&CivCardName::Medicine)
        }
        _ => true,
    }
}

pub fn compute_famine_grain_reduction(grain_count: usize, has_pottery: bool) -> i32 {
    if has_pottery {
        (grain_count * 4) as i32
    } else {
        0
    }
}

pub fn compute_civil_war_victim_selection(civ_cards: &PlayerCivilizationCards) -> usize {
    let mut base = 15;
    if civ_cards.owns(&CivCardName::Music) {
        base += 5;
    }
    if civ_cards.owns(&CivCardName::DramaAndPoetry) {
        base += 5;
    }
    if civ_cards.owns(&CivCardName::Democracy) {
        base += 10;
    }
    base
}
