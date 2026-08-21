use bevy::prelude::{Entity, Reflect};

/// State for resolving Flood (rule 30.51).
///
/// The flood plain with the most primary victim unit points is chosen.
/// Primary victim loses max 17 pts (7 with Engineering) from that flood plain.
/// Secondary victims on the same flood plain collectively lose 10 pts (auto-distributed).
/// If no flood plain has victim units: one coastal city is eliminated (reduced with Engineering).
#[derive(Debug, Clone, Default, Reflect)]
pub struct FloodState {
    pub phase: FloodPhase,
    /// The flood plain area where the flood occurs.
    pub flood_plain_area: Option<Entity>,
    /// How many unit points the primary victim loses (capped at 17, or 7 with Engineering).
    pub primary_max_loss: i32,
    pub has_engineering: bool,
    /// The coastal city picked for 30.514, once chosen.
    pub fallback_city: Option<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum FloodPhase {
    #[default]
    FindFloodPlain,
    ApplyPrimaryLoss,
    ApplySecondaryLoss,
    FallbackCoastalCity,
    /// Waiting for a human victim to pick which coastal city goes (30.514).
    SelectFallbackCity,
    Complete,
}

impl FloodState {
    pub fn new() -> Self {
        Self {
            primary_max_loss: 17,
            ..Default::default()
        }
    }

    pub fn with_engineering(mut self) -> Self {
        self.has_engineering = true;
        self.primary_max_loss = 7;
        self
    }
}

/// Rule 30.512: allocates the 10-point secondary Flood loss among secondary
/// victims on the flood plain. "The primary victim divides the 10-point loss
/// among secondary victims" -- `primary_choice`, when `Some`, is that
/// decision: a list of `(secondary_victim, requested_points)` pairs, each
/// clamped to that player's `available` tokens and to the remaining total
/// budget, processed in order. Unlisted secondary victims lose nothing.
///
/// When `primary_choice` is `None` (no human choice made -- AI auto-play, or
/// no UI wired up yet), falls back to an even split across all secondary
/// victims, still respecting each player's `available` cap.
///
/// Special case (30.512): if the secondary victims' combined available
/// tokens total 10 or fewer, there is no allocation decision to make at all
/// -- "all those players are automatically secondary victims and all their
/// units are eliminated," so every available token is taken regardless of
/// `primary_choice`.
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

    /// Rule 30.511: primary victim loses a maximum of 17 unit points from the flood plain.
    #[test]
    fn base_primary_loss_is_capped_at_17() {
        let state = FloodState::new();
        assert_eq!(state.primary_max_loss, 17);
        assert!(!state.has_engineering);
    }

    /// Rule 30.515: an Engineering-holding primary (or secondary) victim's loss is
    /// capped at 7 instead of 17.
    #[test]
    fn engineering_caps_primary_loss_at_7() {
        let state = FloodState::new().with_engineering();
        assert_eq!(state.primary_max_loss, 7);
        assert!(state.has_engineering);
    }

    // ── Rule 30.512: secondary loss allocation ─────────────────────────────

    use crate::test_utils::create_test_entity;

    #[test]
    fn no_choice_falls_back_to_even_split_capped_by_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // 20 available each, split 10 evenly -> 5 each.
        let result = allocate_secondary_loss(&[(a, 20), (b, 20)], 10, None);
        assert_eq!(result, vec![(a, 5), (b, 5)]);
    }

    #[test]
    fn even_split_respects_per_player_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // a only has 2 available; the even-split target (5) exceeds it, so a
        // loses only 2 -- the other 8 points don't spill over to b in the
        // fallback (no primary-victim decision is being modeled here, just
        // "don't take more than a player has").
        let result = allocate_secondary_loss(&[(a, 2), (b, 20)], 10, None);
        assert_eq!(result, vec![(a, 2), (b, 5)]);
    }

    #[test]
    fn primary_choice_is_honored_and_clamped_by_availability() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Primary victim wants to take all 10 from a, but a only has 6.
        let choice = [(a, 10), (b, 0)];
        let result = allocate_secondary_loss(&[(a, 6), (b, 20)], 10, Some(&choice));
        assert_eq!(result, vec![(a, 6), (b, 0)]);
    }

    #[test]
    fn primary_choice_cannot_exceed_the_total_budget() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Requesting 8+8=16 against a budget of 10 -- a is processed first and
        // gets its full 8, leaving only 2 for b regardless of what b asked for.
        let choice = [(a, 8), (b, 8)];
        let result = allocate_secondary_loss(&[(a, 20), (b, 20)], 10, Some(&choice));
        assert_eq!(result, vec![(a, 8), (b, 2)]);
    }

    #[test]
    fn combined_available_at_or_below_total_eliminates_everyone_regardless_of_choice() {
        let a = create_test_entity();
        let b = create_test_entity();
        // Only 4 + 3 = 7 available, under the 10-point budget -- rule 30.512
        // says all secondary victims automatically lose everything, and a
        // primary_choice (even an empty one) doesn't change that.
        let choice: [(Entity, usize); 0] = [];
        let result = allocate_secondary_loss(&[(a, 4), (b, 3)], 10, Some(&choice));
        assert_eq!(result, vec![(a, 4), (b, 3)]);
    }

    #[test]
    fn no_secondary_players_allocates_nothing() {
        let result = allocate_secondary_loss(&[], 10, None);
        assert_eq!(result, Vec::new());
    }
}
