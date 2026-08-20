use bevy::prelude::{Entity, Reflect};

/// State for resolving Epidemic (rule 30.61).
///
/// Default primary loss: 16 unit points (30.611).
/// Default secondary loss: 25 unit points total, ≤10 per player (30.611).
///
/// Modifiers (cumulative, 30.615):
/// - Medicine     : primary loss −8 (30.613); secondary loss −5 for Medicine holders
/// - Road Building: primary/secondary loss +5 (30.614)
#[derive(Debug, Clone, Default, Reflect)]
pub struct EpidemicState {
    pub phase: EpidemicPhase,
    pub primary_loss: i32,
    /// Total unit points to distribute among secondary victims.
    pub secondary_loss: i32,
    pub has_medicine: bool,
    pub has_road_building: bool,
    pub immune_player: Option<Entity>,
    /// Points still owed after cities have been spent against the primary
    /// loss (rule 30.612). Carried across frames because the token half of
    /// the loss can pause for a human's selection, and the city half must not
    /// be applied twice when it resumes.
    pub primary_tokens_remaining: i32,
    /// Whether the secondary loss has already been divided among the victims.
    /// The division is a one-time decision (and may itself pause for a human
    /// primary victim), while applying it can pause again per victim.
    pub secondary_divided: bool,
    /// Remaining secondary work list: (victim, unit points still owed).
    /// Entries are dropped as they are settled so a pause never re-charges
    /// anyone.
    pub secondary_allocations: Vec<(Entity, i32)>,
    /// Victims whose cities have already been spent against their share
    /// (30.612); their entry in `secondary_allocations` now holds only the
    /// token remainder.
    pub secondary_cities_spent: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum EpidemicPhase {
    #[default]
    ComputeEffects,
    ApplyPrimaryLoss,
    ApplySecondaryLosses,
    Complete,
}

impl EpidemicState {
    pub fn new() -> Self {
        Self {
            primary_loss: 16,
            secondary_loss: 25,
            ..Default::default()
        }
    }

    /// Medicine: primary victim loses 8 fewer unit points (30.613).
    /// Secondary victims who hold Medicine lose 5 fewer — handled during secondary resolution.
    pub fn with_medicine(mut self) -> Self {
        self.has_medicine = true;
        self.primary_loss = (self.primary_loss - 8).max(0);
        self
    }

    /// Road Building: primary (and secondary) victim loses 5 additional unit points (30.614).
    pub fn with_road_building(mut self) -> Self {
        self.has_road_building = true;
        self.primary_loss += 5;
        self
    }

    pub fn with_immune_player(mut self, player: Entity) -> Self {
        self.immune_player = Some(player);
        self
    }
}

/// Rule 30.612: "A nation removing tokens due to Epidemic must leave at least
/// one token in each affected area." Given the current population count in
/// each of a player's areas (in the order losses should be applied) and a
/// total number of points still to remove, returns how many tokens to take
/// from each area, never draining an area to zero. Areas already at 0 or 1
/// token contribute nothing. Applies to both primary and secondary Epidemic
/// losses (the rule doesn't distinguish).
///
/// Note: the companion sub-rule "cities eliminated by Epidemic are replaced
/// by at least one token, so cities count as a maximum of 4 unit points" is
/// NOT modeled here -- Epidemic's loss resolution (`remove_unit_points` in
/// resolve_calamities_systems.rs) only ever touches plain `Population`
/// tokens and never enumerates or eliminates `BuiltCity` entities, so there
/// is no city-point-cap to apply yet. Giving Epidemic real city-awareness is
/// a larger, separate feature; flagged in docs/outline.md.
pub fn allocate_removal_leaving_one_per_area(areas_and_counts: &[usize], points: usize) -> Vec<usize> {
    let mut remaining = points;
    let mut result = Vec::with_capacity(areas_and_counts.len());
    for &count in areas_and_counts {
        if remaining == 0 {
            result.push(0);
            continue;
        }
        let removable = count.saturating_sub(1);
        let take = removable.min(remaining);
        result.push(take);
        remaining -= take;
    }
    result
}

/// Rule 30.611: allocates the 25-point secondary Epidemic loss among secondary
/// victims sharing an area with the primary victim, each capped at 10 points
/// (5 if that victim holds Medicine, rule 30.613 -- callers pass that reduced
/// cap in as the victim's `available`). "The [primary] victim... determines
/// how the total loss... is divided" -- `primary_choice`, when `Some`, is
/// that decision: a list of `(secondary_victim, requested_points)` pairs, each
/// clamped to that player's `available` cap and to the remaining total
/// budget, processed in order. Unlisted secondary victims lose nothing.
///
/// When `primary_choice` is `None` (no human choice made -- AI auto-play, or
/// no UI wired up yet), falls back to an even split across all secondary
/// victims, still respecting each player's `available` cap.
///
/// Special case (30.611): if the secondary victims' combined caps total 25
/// or fewer, there is no allocation decision to make at all -- every
/// available point is taken regardless of `primary_choice`.
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
    use crate::test_utils::create_test_entity;

    /// Rule 30.611: primary victim loses 16 unit points; secondary total 25, max 10/player.
    #[test]
    fn base_losses_match_rule_30_611() {
        let state = EpidemicState::new();
        assert_eq!(state.primary_loss, 16);
        assert_eq!(state.secondary_loss, 25);
    }

    /// Rule 30.613: Medicine reduces the primary victim's loss by 8.
    #[test]
    fn medicine_reduces_primary_loss_by_8() {
        let state = EpidemicState::new().with_medicine();
        assert_eq!(state.primary_loss, 8); // 16 - 8
    }

    /// Rule 30.614: Road Building adds 5 to the primary victim's loss.
    #[test]
    fn road_building_adds_5_to_primary_loss() {
        let state = EpidemicState::new().with_road_building();
        assert_eq!(state.primary_loss, 21); // 16 + 5
    }

    /// Rule 30.615: Medicine and Road Building are cumulative.
    #[test]
    fn medicine_and_road_building_are_cumulative() {
        let state = EpidemicState::new().with_medicine().with_road_building();
        assert_eq!(state.primary_loss, 13); // 16 - 8 + 5
    }

    /// Medicine cannot push the primary loss below zero.
    #[test]
    fn medicine_does_not_go_negative() {
        // Simulate an already-reduced loss going further negative.
        let mut state = EpidemicState::new();
        state.primary_loss = 5;
        state = state.with_medicine();
        assert_eq!(state.primary_loss, 0);
    }

    // ── Rule 30.612: leave at least one token per area ─────────────────────

    #[test]
    fn leaves_one_token_in_each_area_even_with_surplus_points_to_remove() {
        // Areas with 3, 1, and 5 tokens; plenty of points to drain them fully
        // if the cap didn't apply.
        let taken = allocate_removal_leaving_one_per_area(&[3, 1, 5], 100);
        assert_eq!(taken, vec![2, 0, 4]); // count - 1 each, area with 1 token untouched
    }

    #[test]
    fn does_not_take_more_than_the_points_budget() {
        let taken = allocate_removal_leaving_one_per_area(&[10, 10], 3);
        assert_eq!(taken, vec![3, 0]);
        assert_eq!(taken.iter().sum::<usize>(), 3);
    }

    #[test]
    fn empty_and_single_token_areas_contribute_nothing() {
        let taken = allocate_removal_leaving_one_per_area(&[0, 1], 50);
        assert_eq!(taken, vec![0, 0]);
    }

    #[test]
    fn zero_points_takes_nothing() {
        let taken = allocate_removal_leaving_one_per_area(&[5, 5], 0);
        assert_eq!(taken, vec![0, 0]);
    }

    // ── Rule 30.611: secondary loss allocation ──────────────────────────────

    #[test]
    fn no_choice_falls_back_to_even_split_capped_by_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // 10 available (cap) each, split 25 evenly -> ceil(25/2)=13, each capped at 10.
        let result = allocate_secondary_loss(&[(a, 10), (b, 10)], 25, None);
        assert_eq!(result, vec![(a, 10), (b, 10)]);
    }

    #[test]
    fn even_split_respects_per_player_cap() {
        let a = create_test_entity();
        let b = create_test_entity();
        // a is a Medicine holder capped at 5; b has no Medicine, capped at 10.
        // Combined 15 <= 25, so this hits the "everyone loses their cap" special case.
        let result = allocate_secondary_loss(&[(a, 5), (b, 10)], 25, None);
        assert_eq!(result, vec![(a, 5), (b, 10)]);
    }

    #[test]
    fn primary_choice_is_honored_and_clamped_by_cap() {
        let a = create_test_entity();
        let b = create_test_entity();
        let c = create_test_entity();
        // Combined caps (10+10+10=30) exceed the 25 budget, so there IS a
        // decision to make. Primary victim wants to take 15 from a, but a's
        // cap is only 10.
        let choice = [(a, 15), (b, 0), (c, 0)];
        let result = allocate_secondary_loss(&[(a, 10), (b, 10), (c, 10)], 25, Some(&choice));
        assert_eq!(result, vec![(a, 10), (b, 0), (c, 0)]);
    }

    #[test]
    fn primary_choice_cannot_exceed_the_total_budget() {
        let a = create_test_entity();
        let b = create_test_entity();
        let c = create_test_entity();
        // Requesting 10+10+10=30 against a budget of 25 -- a and b are
        // processed first and get their full 10 each, leaving only 5 for c.
        let choice = [(a, 10), (b, 10), (c, 10)];
        let result = allocate_secondary_loss(&[(a, 10), (b, 10), (c, 10)], 25, Some(&choice));
        assert_eq!(result, vec![(a, 10), (b, 10), (c, 5)]);
    }

    /// Rule 30.613: a Medicine holder's cap is 5 instead of 10.
    #[test]
    fn medicine_reduced_cap_is_honored_in_choice() {
        let a = create_test_entity();
        let b = create_test_entity();
        let c = create_test_entity();
        // a holds Medicine (cap 5); wants to take 8 from it, clamped to 5.
        // Combined caps (5+10+15=30) exceed the 25 budget, so there IS a
        // decision to make.
        let choice = [(a, 8), (b, 0), (c, 0)];
        let result = allocate_secondary_loss(&[(a, 5), (b, 10), (c, 15)], 25, Some(&choice));
        assert_eq!(result, vec![(a, 5), (b, 0), (c, 0)]);
    }

    #[test]
    fn combined_available_at_or_below_total_eliminates_everyone_regardless_of_choice() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Only 5 (Medicine) + 10 = 15 available, under the 25-point budget --
        // rule 30.611 says all secondary victims automatically lose up to
        // their cap, and a primary_choice (even an empty one) doesn't change that.
        let choice: [(Entity, usize); 0] = [];
        let result = allocate_secondary_loss(&[(a, 5), (b, 10)], 25, Some(&choice));
        assert_eq!(result, vec![(a, 5), (b, 10)]);
    }

    #[test]
    fn no_secondary_players_allocates_nothing() {
        let result = allocate_secondary_loss(&[], 25, None);
        assert_eq!(result, Vec::new());
    }
}
