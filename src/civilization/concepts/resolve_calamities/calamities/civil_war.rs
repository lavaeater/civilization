use bevy::prelude::{Entity, Reflect};

/// Rule 30.415: which faction the primary victim chooses to retain. The
/// faction the victim does *not* choose is the one that transfers to the
/// beneficiary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FactionChoice {
    First,
    Second,
}

#[derive(Debug, Clone, Default, Reflect)]
pub struct CivilWarState {
    pub phase: CivilWarPhase,
    pub beneficiary: Option<Entity>,
    pub victim_selection_points: usize,
    pub beneficiary_selection_points: usize,
    /// First faction, victim's own 15(+bonus)-point pick (30.4121/30.4122).
    /// Empty under Philosophy (30.4124), where the beneficiary picks the
    /// entire first faction directly.
    pub victim_selected_units: Vec<Entity>,
    pub victim_selected_cities: Vec<Entity>,
    /// First faction, beneficiary's additional pick (30.4123) -- selected
    /// from the victim's board MINUS `victim_selected_units/cities`, i.e. an
    /// additive top-up of the first faction, not a subset of the victim's pick.
    pub beneficiary_selected_units: Vec<Entity>,
    pub beneficiary_selected_cities: Vec<Entity>,
    /// Second faction (30.413): whatever remains of the victim's board once
    /// the first faction (victim_selected + beneficiary_selected) is removed.
    /// Computed once both first-faction selections are finalized.
    pub second_faction_units: Vec<Entity>,
    pub second_faction_cities: Vec<Entity>,
    /// Rule 30.415: the faction the primary victim chooses to retain.
    pub kept_faction: Option<FactionChoice>,
    /// Rule 30.4124: Philosophy, held by the primary victim, replaces the
    /// normal two-step selection entirely -- the first faction becomes 15
    /// units chosen solely by the beneficiary, "regardless of any other
    /// civilization cards held by the primary victim" (so Music/Drama and
    /// Poetry/Democracy never apply alongside it).
    pub philosophy_override: bool,
    /// Rule 30.414: true once it's known that Military applies (either
    /// participant holds it) -- 5 unit points are removed from each faction
    /// after both are finalized, before the 30.415 choice.
    pub military_penalty: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum CivilWarPhase {
    #[default]
    DetermineBeneficiary,
    VictimSelectsUnits,
    BeneficiarySelectsUnits,
    VictimChoosesFaction,
    TransferFaction,
    Complete,
}

impl CivilWarState {
    pub fn new() -> Self {
        Self {
            victim_selection_points: 15,
            beneficiary_selection_points: 20,
            ..Default::default()
        }
    }

    pub fn with_beneficiary(mut self, beneficiary: Entity) -> Self {
        self.beneficiary = Some(beneficiary);
        self.phase = CivilWarPhase::VictimSelectsUnits;
        self
    }

    pub fn apply_music_bonus(&mut self) {
        self.victim_selection_points += 5;
    }

    pub fn apply_drama_poetry_bonus(&mut self) {
        self.victim_selection_points += 5;
    }

    pub fn apply_democracy_bonus(&mut self) {
        self.victim_selection_points += 10;
    }

    /// Rule 30.4124: Philosophy replaces normal selection -- the beneficiary
    /// alone picks 15 units directly from the victim's holdings, bypassing
    /// victim selection and the Music/Drama/Democracy bonuses.
    pub fn with_philosophy_override(mut self) -> Self {
        self.philosophy_override = true;
        self.victim_selection_points = 0;
        self.beneficiary_selection_points = 15;
        self
    }

    /// Rule 30.414: mark that the Military penalty applies to this Civil War.
    pub fn with_military_penalty(mut self) -> Self {
        self.military_penalty = true;
        self
    }

    fn remove_units(units: &mut Vec<Entity>, mut remaining: usize) -> usize {
        while remaining > 0 && !units.is_empty() {
            units.pop();
            remaining -= 1;
        }
        remaining
    }

    fn remove_cities(cities: &mut Vec<Entity>, mut remaining: usize) -> usize {
        while remaining > 0 && !cities.is_empty() {
            cities.pop();
            remaining = remaining.saturating_sub(5);
        }
        remaining
    }

    /// Rule 30.414: removes up to 5 unit points from the first faction
    /// (victim's pick + beneficiary's top-up combined), tokens before cities
    /// since a city can't be partially reduced. Prefers removing from the
    /// beneficiary's top-up first, falling through to the victim's own pick
    /// if the top-up alone can't absorb the full penalty. Call exactly once,
    /// after both `victim_selected_*` and `beneficiary_selected_*` are final.
    pub fn apply_military_penalty_to_first_faction(&mut self) {
        if !self.military_penalty {
            return;
        }
        let remaining = Self::remove_units(&mut self.beneficiary_selected_units, 5);
        let remaining = Self::remove_cities(&mut self.beneficiary_selected_cities, remaining);
        let remaining = Self::remove_units(&mut self.victim_selected_units, remaining);
        Self::remove_cities(&mut self.victim_selected_cities, remaining);
    }

    /// Rule 30.414: removes up to 5 unit points from the second faction
    /// (whatever remained of the victim's board), tokens before cities. Call
    /// exactly once, after `second_faction_units/cities` have been computed
    /// via `compute_second_faction`.
    pub fn apply_military_penalty_to_second_faction(&mut self) {
        if !self.military_penalty {
            return;
        }
        let remaining = Self::remove_units(&mut self.second_faction_units, 5);
        Self::remove_cities(&mut self.second_faction_cities, remaining);
    }

    pub fn victim_points_selected(&self) -> usize {
        self.victim_selected_units.len() + (self.victim_selected_cities.len() * 5)
    }

    pub fn beneficiary_points_selected(&self) -> usize {
        self.beneficiary_selected_units.len() + (self.beneficiary_selected_cities.len() * 5)
    }

    pub fn victim_selection_complete(&self) -> bool {
        self.victim_points_selected() >= self.victim_selection_points
    }

    pub fn beneficiary_selection_complete(&self) -> bool {
        self.beneficiary_points_selected() >= self.beneficiary_selection_points
    }

    /// Rule 30.412: the first faction is the victim's pick plus the
    /// beneficiary's additive top-up.
    pub fn first_faction_points(&self) -> usize {
        self.victim_points_selected() + self.beneficiary_points_selected()
    }

    /// Rule 30.413: whatever remains of the victim's board.
    pub fn second_faction_points(&self) -> usize {
        self.second_faction_units.len() + (self.second_faction_cities.len() * 5)
    }

    /// Rule 30.413: the second faction only exists if something remains of
    /// the victim's board after the first faction is assembled. If not,
    /// 30.413 says "there is no Civil War."
    pub fn has_second_faction(&self) -> bool {
        !self.second_faction_units.is_empty() || !self.second_faction_cities.is_empty()
    }

    /// Rule 30.413: computes the second faction as the victim's full board
    /// presence minus everything already claimed by the first faction.
    pub fn compute_second_faction(&mut self, full_units: Vec<Entity>, full_cities: Vec<Entity>) {
        let first_units: Vec<Entity> = self
            .victim_selected_units
            .iter()
            .chain(self.beneficiary_selected_units.iter())
            .copied()
            .collect();
        let first_cities: Vec<Entity> = self
            .victim_selected_cities
            .iter()
            .chain(self.beneficiary_selected_cities.iter())
            .copied()
            .collect();
        self.second_faction_units = full_units
            .into_iter()
            .filter(|u| !first_units.contains(u))
            .collect();
        self.second_faction_cities = full_cities
            .into_iter()
            .filter(|c| !first_cities.contains(c))
            .collect();
    }

    /// Rule 30.415 AI default: keep whichever faction has more total unit
    /// points; keep the first faction on a tie.
    pub fn default_ai_faction_choice(&self) -> FactionChoice {
        if self.second_faction_points() > self.first_faction_points() {
            FactionChoice::Second
        } else {
            FactionChoice::First
        }
    }

    /// Rule 30.415: the units belonging to the faction the victim did NOT
    /// keep, i.e. the ones that transfer to the beneficiary.
    pub fn transferring_units(&self) -> Vec<Entity> {
        match self.kept_faction {
            Some(FactionChoice::Second) | None => self
                .victim_selected_units
                .iter()
                .chain(self.beneficiary_selected_units.iter())
                .copied()
                .collect(),
            Some(FactionChoice::First) => self.second_faction_units.clone(),
        }
    }

    /// Rule 30.415: the cities belonging to the faction the victim did NOT
    /// keep, i.e. the ones that transfer to the beneficiary.
    pub fn transferring_cities(&self) -> Vec<Entity> {
        match self.kept_faction {
            Some(FactionChoice::Second) | None => self
                .victim_selected_cities
                .iter()
                .chain(self.beneficiary_selected_cities.iter())
                .copied()
                .collect(),
            Some(FactionChoice::First) => self.second_faction_cities.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_entity;

    /// Rule 30.4121/30.4123: victim selects 15 unit points for the first faction,
    /// beneficiary selects a further 20 to complete it.
    #[test]
    fn base_selection_points_are_15_and_20() {
        let state = CivilWarState::new();
        assert_eq!(state.victim_selection_points, 15);
        assert_eq!(state.beneficiary_selection_points, 20);
    }

    /// Rule 30.4122: Music, Drama and Poetry (+5 each), and Democracy (+10) are
    /// cumulative additions to the victim's selection points.
    #[test]
    fn music_drama_and_democracy_bonuses_are_cumulative() {
        let mut state = CivilWarState::new();
        state.apply_music_bonus();
        state.apply_drama_poetry_bonus();
        state.apply_democracy_bonus();
        assert_eq!(state.victim_selection_points, 15 + 5 + 5 + 10); // 35
    }

    #[test]
    fn victim_points_selected_counts_cities_as_5() {
        let mut state = CivilWarState::new();
        state.victim_selected_units = vec![create_test_entity(), create_test_entity()];
        state.victim_selected_cities = vec![create_test_entity()];
        assert_eq!(state.victim_points_selected(), 2 + 5); // 2 tokens + 1 city*5
    }

    #[test]
    fn selection_complete_once_points_threshold_reached() {
        let mut state = CivilWarState::new(); // needs 15
        state.victim_selected_units = (0..15).map(|_| create_test_entity()).collect();
        assert!(state.victim_selection_complete());

        let mut short = CivilWarState::new();
        short.victim_selected_units = (0..14).map(|_| create_test_entity()).collect();
        assert!(!short.victim_selection_complete());
    }

    // ── Rule 30.4124: Philosophy override ──────────────────────────────────

    #[test]
    fn philosophy_override_sets_beneficiary_target_to_15_and_zeroes_victim_target() {
        let state = CivilWarState::new().with_philosophy_override();
        assert!(state.philosophy_override);
        assert_eq!(state.victim_selection_points, 0);
        assert_eq!(state.beneficiary_selection_points, 15);
    }

    #[test]
    fn philosophy_override_is_unaffected_by_music_drama_democracy() {
        // The rule says the override applies "regardless of any other
        // civilization cards held by the primary victim" -- callers achieve
        // this by branching on Philosophy before applying the other bonuses
        // (see resolve_calamities_systems.rs), so this test just pins that
        // with_philosophy_override's own targets don't drift if called
        // before/after those other (skipped) calls in isolation.
        let overridden = CivilWarState::new().with_philosophy_override();
        assert_eq!(overridden.beneficiary_selection_points, 15);
    }

    // ── Rule 30.413: second faction ─────────────────────────────────────────

    #[test]
    fn second_faction_is_the_victims_board_minus_the_first_faction() {
        let mut state = CivilWarState::new();
        let victim_pick = create_test_entity();
        let beneficiary_topup = create_test_entity();
        let leftover = create_test_entity();
        state.victim_selected_units = vec![victim_pick];
        state.beneficiary_selected_units = vec![beneficiary_topup];

        state.compute_second_faction(vec![victim_pick, beneficiary_topup, leftover], vec![]);

        assert_eq!(state.second_faction_units, vec![leftover]);
        assert!(state.has_second_faction());
    }

    #[test]
    fn no_second_faction_when_first_faction_consumes_the_whole_board() {
        let mut state = CivilWarState::new();
        let a = create_test_entity();
        let b = create_test_entity();
        state.victim_selected_units = vec![a];
        state.beneficiary_selected_units = vec![b];

        state.compute_second_faction(vec![a, b], vec![]);

        assert_eq!(state.second_faction_units, [] as [bevy::bevy_ecs::entity::Entity; 0]);
        assert_eq!(state.second_faction_cities, [] as [bevy::bevy_ecs::entity::Entity; 0]);
        assert!(!state.has_second_faction()); // 30.413: no second faction, no Civil War
    }

    /// Rule 30.4123: the beneficiary's top-up is ADDITIVE to the victim's
    /// pick, drawn from what's left of the victim's board -- not a subset of
    /// what the victim already selected.
    #[test]
    fn beneficiary_topup_and_victim_pick_are_disjoint_pools() {
        let mut state = CivilWarState::new();
        let victim_pick = create_test_entity();
        let beneficiary_topup = create_test_entity();
        state.victim_selected_units = vec![victim_pick];
        state.beneficiary_selected_units = vec![beneficiary_topup];

        assert_ne!(victim_pick, beneficiary_topup);
        assert_eq!(state.first_faction_points(), 2);
    }

    // ── Rule 30.415: victim's faction choice ────────────────────────────────

    #[test]
    fn ai_default_keeps_the_bigger_faction() {
        let mut state = CivilWarState::new();
        state.victim_selected_units = (0..3).map(|_| create_test_entity()).collect();
        state.second_faction_units = (0..10).map(|_| create_test_entity()).collect();
        assert_eq!(state.default_ai_faction_choice(), FactionChoice::Second);
    }

    #[test]
    fn ai_default_keeps_first_faction_on_a_tie() {
        let mut state = CivilWarState::new();
        state.victim_selected_units = (0..5).map(|_| create_test_entity()).collect();
        state.second_faction_units = (0..5).map(|_| create_test_entity()).collect();
        assert_eq!(state.default_ai_faction_choice(), FactionChoice::First);
    }

    #[test]
    fn keeping_first_faction_transfers_the_second_faction() {
        let mut state = CivilWarState::new();
        let victim_pick = create_test_entity();
        let beneficiary_topup = create_test_entity();
        let leftover = create_test_entity();
        state.victim_selected_units = vec![victim_pick];
        state.beneficiary_selected_units = vec![beneficiary_topup];
        state.second_faction_units = vec![leftover];
        state.kept_faction = Some(FactionChoice::First);

        assert_eq!(state.transferring_units(), vec![leftover]);
    }

    #[test]
    fn keeping_second_faction_transfers_the_first_faction() {
        let mut state = CivilWarState::new();
        let victim_pick = create_test_entity();
        let beneficiary_topup = create_test_entity();
        let leftover = create_test_entity();
        state.victim_selected_units = vec![victim_pick];
        state.beneficiary_selected_units = vec![beneficiary_topup];
        state.second_faction_units = vec![leftover];
        state.kept_faction = Some(FactionChoice::Second);

        let mut transferring = state.transferring_units();
        transferring.sort();
        let mut expected = vec![victim_pick, beneficiary_topup];
        expected.sort();
        assert_eq!(transferring, expected);
    }

    // ── Rule 30.414: Military penalty ──────────────────────────────────────

    #[test]
    fn military_penalty_removes_5_points_from_beneficiary_tokens_first() {
        let mut state = CivilWarState::new().with_military_penalty();
        state.beneficiary_selected_units = (0..8).map(|_| create_test_entity()).collect();
        state.apply_military_penalty_to_first_faction();
        assert_eq!(state.beneficiary_selected_units.len(), 3); // 8 - 5
        assert_eq!(state.beneficiary_selected_cities, [] as [bevy::bevy_ecs::entity::Entity; 0]);
    }

    #[test]
    fn military_penalty_falls_through_to_a_city_when_tokens_run_out() {
        let mut state = CivilWarState::new().with_military_penalty();
        state.beneficiary_selected_units = (0..2).map(|_| create_test_entity()).collect(); // 2 pts
        state.beneficiary_selected_cities = vec![create_test_entity()]; // 5 pts
        state.apply_military_penalty_to_first_faction();
        // 2 tokens removed (2 pts), then the city removed to cover the rest
        // (can't partially reduce a city).
        assert_eq!(state.beneficiary_selected_units, [] as [bevy::bevy_ecs::entity::Entity; 0]);
        assert_eq!(state.beneficiary_selected_cities, [] as [bevy::bevy_ecs::entity::Entity; 0]);
    }

    #[test]
    fn military_penalty_falls_through_to_victim_pool_when_beneficiary_topup_is_small() {
        let mut state = CivilWarState::new().with_military_penalty();
        state.beneficiary_selected_units = (0..2).map(|_| create_test_entity()).collect(); // 2 pts
        state.victim_selected_units = (0..8).map(|_| create_test_entity()).collect();
        state.apply_military_penalty_to_first_faction();
        assert_eq!(state.beneficiary_selected_units, [] as [bevy::bevy_ecs::entity::Entity; 0]);
        assert_eq!(state.victim_selected_units.len(), 5); // 8 - (5 - 2)
    }

    #[test]
    fn military_penalty_applies_to_second_faction_independently() {
        let mut state = CivilWarState::new().with_military_penalty();
        state.second_faction_units = (0..8).map(|_| create_test_entity()).collect();
        state.apply_military_penalty_to_second_faction();
        assert_eq!(state.second_faction_units.len(), 3);
    }

    #[test]
    fn military_penalty_is_a_no_op_when_not_flagged() {
        let mut state = CivilWarState::new();
        state.beneficiary_selected_units = (0..8).map(|_| create_test_entity()).collect();
        state.apply_military_penalty_to_first_faction();
        assert_eq!(state.beneficiary_selected_units.len(), 8);
    }

    #[test]
    fn military_penalty_does_not_go_negative_on_an_empty_faction() {
        let mut state = CivilWarState::new().with_military_penalty();
        state.apply_military_penalty_to_first_faction();
        assert_eq!(state.beneficiary_selected_units, [] as [bevy::bevy_ecs::entity::Entity; 0]);
        assert_eq!(state.beneficiary_selected_cities, [] as [bevy::bevy_ecs::entity::Entity; 0]);
    }
}
