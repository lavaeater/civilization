use bevy::prelude::{Entity, Reflect};

#[derive(Debug, Clone, Default, Reflect)]
pub struct FamineState {
    pub phase: FaminePhase,
    pub primary_loss: i32,
    pub secondary_total: i32,
    pub max_per_secondary: i32,
    pub grain_reduction: i32,
    /// Rule 30.312: how many Grain cards were actually placed face up for
    /// this reduction. Only as many as needed to zero out the loss are used
    /// -- no reason to lock more of a scarce commodity than required. This
    /// is the count `GrainLockedForPurchase` should be set to.
    pub grain_cards_used: usize,
    pub secondary_allocations: Vec<(Entity, i32)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum FaminePhase {
    #[default]
    ComputeLosses,
    SelectSecondaryVictims,
    ApplyPrimaryLoss,
    ApplySecondaryLosses,
    Complete,
}

impl FamineState {
    pub fn new() -> Self {
        Self {
            primary_loss: 10,
            secondary_total: 20,
            max_per_secondary: 8,
            ..Default::default()
        }
    }

    pub fn with_grain_reduction(mut self, grain_count: usize, has_pottery: bool) -> Self {
        if has_pottery && grain_count > 0 {
            // Rule 30.312: cards are "used for this purpose", not simply held --
            // only enough to zero out the current loss are placed face up.
            let loss = self.primary_loss.max(0) as usize;
            let cards_needed = loss.div_ceil(4);
            let used = cards_needed.min(grain_count);
            self.grain_cards_used = used;
            self.grain_reduction = (used * 4) as i32;
            self.primary_loss = (self.primary_loss - self.grain_reduction).max(0);
        }
        self
    }

    pub fn add_secondary_allocation(&mut self, victim: Entity, points: i32) {
        let clamped = points.min(self.max_per_secondary);
        self.secondary_allocations.push((victim, clamped));
    }

    pub fn remaining_secondary_points(&self) -> i32 {
        let allocated: i32 = self.secondary_allocations.iter().map(|(_, p)| *p).sum();
        self.secondary_total - allocated
    }
}

/// Rule 30.311: allocates the 20-point secondary Famine loss among secondary
/// victims sharing an area with the primary victim. "The primary victim
/// decides how these points are distributed" among secondary victims, no
/// more than 8 from any one of them -- `primary_choice`, when `Some`, is
/// that decision: a list of `(secondary_victim, requested_points)` pairs,
/// each clamped to that player's `available` tokens (already folded down to
/// the 8-point-per-player cap by the caller) and to the remaining total
/// budget, processed in order. Unlisted secondary victims lose nothing.
///
/// When `primary_choice` is `None` (no human choice made -- AI auto-play, or
/// no UI wired up yet), falls back to an even split across all secondary
/// victims, still respecting each player's `available` cap.
///
/// Special case: if the secondary victims' combined available tokens total
/// 20 or fewer, there is no allocation decision to make at all -- every
/// available token is taken regardless of `primary_choice`.
pub fn allocate_secondary_loss(
    secondary_players: &[(Entity, usize)],
    total: usize,
    primary_choice: Option<&[(Entity, usize)]>,
) -> Vec<(Entity, usize)> {
    let combined_available: usize = secondary_players.iter().map(|(_, a)| a).sum();
    if combined_available <= total {
        return secondary_players.to_vec();
    }

    if let Some(choice) = primary_choice {
        let mut remaining = total;
        let mut result = Vec::with_capacity(secondary_players.len());
        for &(player, available) in secondary_players {
            let requested = choice
                .iter()
                .find(|(p, _)| *p == player)
                .map_or(0, |(_, pts)| *pts);
            let take = requested.min(available).min(remaining);
            result.push((player, take));
            remaining -= take;
        }
        return result;
    }

    // Even split, each still capped by that player's available tokens.
    let n = secondary_players.len();
    if n == 0 {
        return Vec::new();
    }
    let per_player = total.div_ceil(n);
    let mut remaining = total;
    let mut result = Vec::with_capacity(n);
    for &(player, available) in secondary_players {
        let take = per_player.min(available).min(remaining);
        result.push((player, take));
        remaining -= take;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rule 30.311: primary victim loses 10 unit points; secondary victims lose 20
    /// total, no more than 8 from any one player.
    #[test]
    fn base_losses_match_rule_30_311() {
        let state = FamineState::new();
        assert_eq!(state.primary_loss, 10);
        assert_eq!(state.secondary_total, 20);
        assert_eq!(state.max_per_secondary, 8);
    }

    /// Rule 30.312: Pottery reduces the primary victim's loss by 4 per Grain card held.
    #[test]
    fn pottery_reduces_primary_loss_by_4_per_grain_card() {
        let state = FamineState::new().with_grain_reduction(2, true);
        assert_eq!(state.grain_reduction, 8);
        assert_eq!(state.primary_loss, 2); // 10 - 8
    }

    /// The reduction cannot take the loss below zero.
    #[test]
    fn pottery_reduction_does_not_go_negative() {
        let state = FamineState::new().with_grain_reduction(5, true); // 5*4 = 20 > 10
        assert_eq!(state.primary_loss, 0);
    }

    /// Without Pottery, holding Grain cards has no effect (30.312 requires the card).
    #[test]
    fn no_pottery_means_no_reduction_regardless_of_grain_held() {
        let state = FamineState::new().with_grain_reduction(3, false);
        assert_eq!(state.grain_reduction, 0);
        assert_eq!(state.primary_loss, 10);
    }

    // ── Rule 30.312: only as many Grain cards as needed are "used" ─────────

    /// Holding more Grain than needed to zero the loss: only the needed
    /// count is used/locked, not the whole hand (10 pts needs 3 cards; 2 of
    /// those cover the primary_loss=10 case exactly at the boundary below).
    #[test]
    fn only_as_many_cards_as_needed_are_marked_used() {
        // primary_loss = 10 -> ceil(10/4) = 3 cards fully zero it.
        let state = FamineState::new().with_grain_reduction(10, true);
        assert_eq!(
            state.grain_cards_used, 3,
            "10 held, only 3 needed to zero a loss of 10"
        );
        assert_eq!(state.primary_loss, 0);
    }

    /// Holding fewer than needed: all held cards are used (can't use more than owned).
    #[test]
    fn grain_cards_used_is_capped_by_cards_held() {
        let state = FamineState::new().with_grain_reduction(2, true);
        assert_eq!(state.grain_cards_used, 2);
    }

    #[test]
    fn no_pottery_uses_no_grain_cards() {
        let state = FamineState::new().with_grain_reduction(5, false);
        assert_eq!(state.grain_cards_used, 0);
    }

    // ── Rule 30.311: secondary loss allocation ─────────────────────────────

    use crate::test_utils::create_test_entity;

    #[test]
    fn no_choice_falls_back_to_even_split_capped_by_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // 20 available each, split 20 evenly -> 10 each.
        let result = allocate_secondary_loss(&[(a, 20), (b, 20)], 20, None);
        assert_eq!(result, vec![(a, 10), (b, 10)]);
    }

    #[test]
    fn even_split_respects_per_player_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // a only has 2 available; the even-split target (10) exceeds it, so a
        // loses only 2 -- the other 8 points don't spill over to b in the
        // fallback (no primary-victim decision is being modeled here, just
        // "don't take more than a player has").
        let result = allocate_secondary_loss(&[(a, 2), (b, 20)], 20, None);
        assert_eq!(result, vec![(a, 2), (b, 10)]);
    }

    #[test]
    fn primary_choice_is_honored_and_clamped_by_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Primary victim wants to take all 20 from a, but a only has 8
        // (the per-player cap folded into `available` by the caller).
        let choice = [(a, 20), (b, 0)];
        let result = allocate_secondary_loss(&[(a, 8), (b, 20)], 20, Some(&choice));
        assert_eq!(result, vec![(a, 8), (b, 0)]);
    }

    #[test]
    fn primary_choice_cannot_exceed_the_total_budget() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Requesting 8+8=16 against a budget of 20 is fine, but here a and b
        // both request their full 8-point cap and a small remainder -- a is
        // processed first and gets its full 12 (its available cap), leaving
        // only 8 for b regardless of what b asked for.
        let choice = [(a, 12), (b, 12)];
        let result = allocate_secondary_loss(&[(a, 20), (b, 20)], 20, Some(&choice));
        assert_eq!(result, vec![(a, 12), (b, 8)]);
    }

    #[test]
    fn combined_available_at_or_below_total_eliminates_everyone_regardless_of_choice() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Only 8 + 6 = 14 available (already capped at 8/player), under the
        // 20-point budget -- everyone automatically loses everything, and a
        // primary_choice (even an empty one) doesn't change that.
        let choice: [(Entity, usize); 0] = [];
        let result = allocate_secondary_loss(&[(a, 8), (b, 6)], 20, Some(&choice));
        assert_eq!(result, vec![(a, 8), (b, 6)]);
    }

    #[test]
    fn no_secondary_players_allocates_nothing() {
        let result = allocate_secondary_loss(&[], 20, None);
        assert_eq!(result, Vec::new());
    }
}
