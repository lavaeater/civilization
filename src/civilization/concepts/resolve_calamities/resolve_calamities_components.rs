use bevy::prelude::{Component, Entity, Reflect, ReflectComponent};
use crate::civilization::TradeCard;

/// Marks a player that holds Monotheism and must process their conversions
/// at the end of the Resolve Calamities phase (rule 32.94).
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct NeedsMonotheismConversion;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct NeedsCalamityResolution;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CalamityVictim {
    pub calamity: TradeCard,
    pub traded_by: Option<Entity>,
}

impl CalamityVictim {
    pub fn new(calamity: TradeCard, traded_by: Option<Entity>) -> Self {
        Self { calamity, traded_by }
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PendingCalamities {
    pub calamities: Vec<(TradeCard, Option<Entity>)>,
}

impl PendingCalamities {
    pub fn new(calamities: Vec<(TradeCard, Option<Entity>)>) -> Self {
        Self { calamities }
    }

    pub fn count(&self) -> usize {
        self.calamities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.calamities.is_empty()
    }
}

/// Rule 30.312: Grain cards used for a Famine/Pottery reduction are placed
/// face up and "may not be used to acquire civilization cards" until the
/// following turn. Set to the number of cards so used when Famine's primary
/// loss is applied; cleared at the start of the next turn (`OnEnter(GameActivity::PopulationExpansion)`).
///
/// Enforced wherever Grain cards are offered or spent for a civ-card purchase:
/// `build_payment_ui`/`handle_payment_adjust` (human UI, civ_cards/systems.rs),
/// `process_civ_card_purchase` (authoritative commit, same file), and
/// `compute_ai_payment` (AI selection, stupid_ai/stupid_ai_systems.rs) all
/// call [`usable_grain_count`] to cap how many Grain cards are visible/
/// spendable, rather than each reimplementing the subtraction.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct GrainLockedForPurchase(pub usize);

/// Rule 30.312: how many of a player's `held` Grain cards are actually usable
/// to pay for a civilization card this turn, given `locked` of them are
/// tied up face-up from a Famine/Pottery reduction. Never negative -- if
/// `locked` somehow exceeds `held` (shouldn't happen; the lock is set from
/// cards that were counted as held at the time), all Grain is simply unusable
/// rather than underflowing.
pub fn usable_grain_count(held: usize, locked: usize) -> usize {
    held.saturating_sub(locked)
}

/// Marks the single, persistent, shared Pirate-nation owner entity created by
/// Piracy (rule 30.91). Unlike Barbarian Hordes' ephemeral owner (despawned
/// at the end of that calamity's resolution), Pirate cities "remain until
/// attacked and destroyed" (30.913), possibly many turns later -- so this
/// entity must survive indefinitely once created. There is only ever one:
/// every Piracy instance across the whole game reuses the same entity
/// (`ensure_pirate_nation` in `resolve_calamities_systems.rs` finds it via
/// this marker before spawning a new one). See `PiracyState`'s doc comment
/// for the full design, including why `start_check_city_support` must
/// explicitly filter this marker out (rule 30.913: no city support required).
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PirateNation;

#[cfg(test)]
mod grain_lock_tests {
    use super::*;

    #[test]
    fn usable_is_held_minus_locked() {
        assert_eq!(usable_grain_count(5, 2), 3);
    }

    #[test]
    fn fully_locked_leaves_nothing_usable() {
        assert_eq!(usable_grain_count(3, 3), 0);
    }

    #[test]
    fn no_lock_leaves_everything_usable() {
        assert_eq!(usable_grain_count(4, 0), 4);
    }

    #[test]
    fn locked_exceeding_held_saturates_to_zero_rather_than_underflowing() {
        assert_eq!(usable_grain_count(2, 5), 0);
    }
}

