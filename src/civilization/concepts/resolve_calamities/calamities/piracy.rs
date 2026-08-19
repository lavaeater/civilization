use bevy::prelude::{Entity, Reflect};

/// State for Piracy (rule 30.91).
///
/// Pirate cities are owned by a lightweight, PERSISTENT (unlike Barbarian
/// Hordes' ephemeral owner -- rule 30.913 says Pirate cities "remain until
/// attacked and destroyed", possibly many turns later) non-`Player` owner
/// entity marked `PirateNation`, found-or-created by `ensure_pirate_nation`
/// in `resolve_calamities_systems.rs`. It carries `CityTokenStock`,
/// `TokenStock`, `PlayerCities`, and `PlayerAreas` -- exactly the components
/// `transfer_city_to_new_owner` and the real Conflict-phase city-combat
/// machinery (`conflict_triggers.rs`) already require of any city owner --
/// so a Pirate city plugs directly into both with zero bespoke code. Lacking
/// `Player`/`Treasury`/`PlayerTradeCards` naturally exempts it from taxation,
/// trading, and (per 24.51's existing attacker/victim trade-card-draw gate)
/// ever having a card drawn from or by it. City support is exempted
/// explicitly in `start_check_city_support` (rule 30.913's "do not require
/// city support" needs an explicit skip, since that query isn't otherwise
/// gated by `With<Player>`).
///
/// KNOWN SIMPLIFICATIONS (documented, not silently dropped -- see
/// docs/outline.md): the trading player's choice of which 2 coastal cities
/// the primary victim loses (30.911), and the primary victim's choice of
/// which player-owned city becomes a Pirate city among an ineligible
/// player's holdings if they have none coastal (30.912's spirit), are
/// auto-selected deterministically rather than built as new interactive UI.
/// The primary-victim-selects-2-secondary-players step (30.912) already had
/// real interactive selection wired in before this pass (`CalamitySelectionState`)
/// and is untouched here. Adjacent-area / "nearest vulnerable" reasoning
/// doesn't apply to Piracy at all (unlike Barbarian Hordes) -- there's no
/// movement, just direct city replacement.
#[derive(Debug, Clone, Default, Reflect)]
pub struct PiracyState {
    pub phase: PiracyPhase,
    pub coastal_cities_to_replace: usize,
    pub selected_cities: Vec<Entity>,
    pub cities_to_replace: Vec<Entity>,
    /// The persistent Pirate-nation owner entity, resolved (found or spawned)
    /// once in `EnsurePirateNation` and reused for every `TransferCityTo` in
    /// `ApplyEffects`. Resolved at least one phase (i.e. one frame) before
    /// it's referenced, so a freshly-spawned entity's components have always
    /// flushed via `Commands` by the time `transfer_city_to_new_owner` reads them.
    pub pirate_nation: Option<Entity>,
    pub immune_player: Option<Entity>,
    /// Cities selected from secondary victims (area entities). Rule 30.912.
    pub secondary_cities: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum PiracyPhase {
    /// Find the existing `PirateNation`-marked entity, or spawn a new one
    /// with its `CityTokenStock`/`TokenStock`/`PlayerCities`/`PlayerAreas`.
    /// There is only ever one Pirate nation for the whole game, shared
    /// across every Piracy calamity instance and every player it affects.
    #[default]
    EnsurePirateNation,
    SelectCoastalCities,
    SelectSecondaryVictims,
    ApplyEffects,
    Complete,
}

impl PiracyState {
    pub fn new() -> Self {
        Self {
            coastal_cities_to_replace: 2,
            ..Default::default()
        }
    }

    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rule 30.911: the primary victim loses two coastal cities.
    #[test]
    fn base_replaces_2_primary_coastal_cities() {
        assert_eq!(PiracyState::new().coastal_cities_to_replace, 2);
    }

    /// The player who traded Piracy to the primary victim may not be a secondary
    /// victim (30.912's trailing clause).
    #[test]
    fn immune_player_is_recorded() {
        let trader = crate::test_utils::create_test_entity();
        let state = PiracyState::new().with_immune_player(trader);
        assert_eq!(state.immune_player, Some(trader));
    }

    /// Rule 30.911/30.913: real Pirate cities (a persistent, shared,
    /// non-`Player` owner entity), not a transfer to a beneficiary player --
    /// fixed alongside the Barbarian Hordes rewrite this session. See the
    /// state struct's doc comment for the full design and `ensure_pirate_nation`
    /// / `advance_piracy` in `resolve_calamities_systems.rs` for the wiring.
    #[test]
    fn new_state_starts_at_ensure_pirate_nation_with_no_nation_resolved_yet() {
        let state = PiracyState::new();
        assert_eq!(state.phase, PiracyPhase::EnsurePirateNation);
        assert!(state.pirate_nation.is_none());
    }
}
