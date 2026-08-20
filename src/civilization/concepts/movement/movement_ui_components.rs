use bevy::platform::collections::HashSet;
use bevy::prelude::*;

/// Resource tracking the current movement selection state for human player
#[derive(Resource, Default, Debug)]
pub struct MovementSelectionState {
    /// The human player currently selecting a move
    pub player: Option<Entity>,
    /// All source areas the player can move from
    pub source_areas: Vec<Entity>,
    /// Index into source_areas for the currently focused source
    pub current_source_index: usize,
    /// Source area for the movement
    pub source_area: Option<Entity>,
    /// Target area for the movement  
    pub target_area: Option<Entity>,
    /// Number of tokens to move (0 = no selection)
    pub token_count: usize,
    /// Maximum tokens that can be moved from source to target
    pub max_tokens: usize,
    /// Whether this is an attack move
    pub is_attack: bool,
    /// Whether attacking a city
    pub is_city_attack: bool,
    /// Source areas the player has chosen to skip
    pub skipped_sources: HashSet<Entity>,
}

impl MovementSelectionState {
    pub fn clear(&mut self) {
        self.player = None;
        self.source_areas.clear();
        self.current_source_index = 0;
        self.source_area = None;
        self.target_area = None;
        self.token_count = 0;
        self.max_tokens = 0;
        self.is_attack = false;
        self.is_city_attack = false;
        self.skipped_sources.clear();
    }

    /// Clears selection state but preserves skipped sources for the rest of the movement phase
    pub fn clear_preserving_skips(&mut self) {
        self.source_area = None;
        self.target_area = None;
        self.token_count = 0;
        self.max_tokens = 0;
        self.is_attack = false;
        self.is_city_attack = false;
    }

    pub fn clear_target(&mut self) {
        self.target_area = None;
        self.token_count = 0;
        self.max_tokens = 0;
        self.is_attack = false;
        self.is_city_attack = false;
    }

    /// Rebuilds `source_areas` after the player's moves were recalculated,
    /// keeping the player where they were instead of throwing them back to the
    /// top of the list.
    ///
    /// `AvailableMoves` is removed and re-inserted after every single move, so
    /// this runs constantly mid-phase. Two things used to jump: the index was
    /// reset to the start each time, and the list itself was rebuilt from a
    /// `HashMap`'s iteration order, so even untouched areas changed places.
    ///
    /// Areas that still have moves keep their previous order; genuinely new
    /// ones are appended. The cursor stays on the same area if it survived,
    /// and otherwise lands on the next area that followed it in the old
    /// order -- moving on from where the player was, never starting over.
    pub fn resync_sources(&mut self, new_sources: &[Entity]) {
        let previous_current = self.current_source();
        let previous_order = std::mem::take(&mut self.source_areas);

        let mut ordered: Vec<Entity> = previous_order
            .iter()
            .copied()
            .filter(|area| new_sources.contains(area))
            .collect();
        for &area in new_sources {
            if !ordered.contains(&area) {
                ordered.push(area);
            }
        }
        self.source_areas = ordered;

        // Drop skips for areas that are no longer offered, so a later area
        // reusing the same entity id can't inherit a stale skip.
        self.skipped_sources
            .retain(|area| self.source_areas.contains(area));

        if self.source_areas.is_empty() {
            self.current_source_index = 0;
            return;
        }

        self.current_source_index = match previous_current {
            // Still available: stay exactly where the player left off.
            Some(current) if self.source_areas.contains(&current) => self
                .source_areas
                .iter()
                .position(|&area| area == current)
                .unwrap_or(0),
            // Finished (or gone): resume from whatever followed it before.
            Some(current) => previous_order
                .iter()
                .position(|&area| area == current)
                .and_then(|previous_index| {
                    previous_order
                        .iter()
                        .cycle()
                        .skip(previous_index + 1)
                        .take(previous_order.len())
                        .find_map(|area| self.source_areas.iter().position(|a| a == area))
                })
                .unwrap_or(0),
            None => 0,
        };

        if self.is_current_skipped() {
            self.advance_to_unskipped();
        }
    }

    pub fn current_source(&self) -> Option<Entity> {
        self.source_areas.get(self.current_source_index).copied()
    }

    pub fn is_current_skipped(&self) -> bool {
        self.current_source()
            .is_some_and(|s| self.skipped_sources.contains(&s))
    }

    pub fn skip_current_source(&mut self) {
        if let Some(source) = self.current_source() {
            self.skipped_sources.insert(source);
            self.clear_target();
            // Auto-advance to next unskipped source if possible
            self.advance_to_unskipped();
        }
    }

    pub fn unskip_current_source(&mut self) {
        if let Some(source) = self.current_source() {
            self.skipped_sources.remove(&source);
        }
    }

    pub fn all_skipped(&self) -> bool {
        !self.source_areas.is_empty()
            && self
                .source_areas
                .iter()
                .all(|s| self.skipped_sources.contains(s))
    }

    fn advance_to_unskipped(&mut self) {
        if self.source_areas.is_empty() {
            return;
        }
        let start = self.current_source_index;
        loop {
            self.current_source_index = (self.current_source_index + 1) % self.source_areas.len();
            if !self.is_current_skipped() || self.current_source_index == start {
                break;
            }
        }
    }

    pub fn next_source(&mut self) {
        if !self.source_areas.is_empty() {
            self.clear_target();
            let start = self.current_source_index;
            loop {
                self.current_source_index =
                    (self.current_source_index + 1) % self.source_areas.len();
                if !self.is_current_skipped() || self.current_source_index == start {
                    break;
                }
            }
        }
    }

    pub fn prev_source(&mut self) {
        if !self.source_areas.is_empty() {
            self.clear_target();
            let start = self.current_source_index;
            loop {
                if self.current_source_index == 0 {
                    self.current_source_index = self.source_areas.len() - 1;
                } else {
                    self.current_source_index -= 1;
                }
                if !self.is_current_skipped() || self.current_source_index == start {
                    break;
                }
            }
        }
    }

    pub fn has_selection(&self) -> bool {
        self.target_area.is_some() && self.token_count > 0
    }

    pub fn select_target(
        &mut self,
        player: Entity,
        source: Entity,
        target: Entity,
        max_tokens: usize,
        is_attack: bool,
        is_city_attack: bool,
    ) {
        self.player = Some(player);
        self.source_area = Some(source);
        self.target_area = Some(target);
        self.max_tokens = max_tokens;
        self.token_count = 1; // Start with 1 token selected
        self.is_attack = is_attack;
        self.is_city_attack = is_city_attack;
    }

    pub fn increment(&mut self) {
        if self.token_count < self.max_tokens {
            self.token_count += 1;
        }
    }

    pub fn decrement(&mut self) {
        if self.token_count > 0 {
            self.token_count -= 1;
        }
        // If count reaches 0, clear the selection
        if self.token_count == 0 {
            self.target_area = None;
            self.source_area = None;
            self.max_tokens = 0;
            self.is_attack = false;
            self.is_city_attack = false;
        }
    }
}

/// Marker for areas that are valid movement sources for the human player
#[derive(Component, Debug)]
pub struct MovementSourceHighlight {
    pub player: Entity,
}

/// Marker for areas that are valid movement targets from a source
#[derive(Component, Debug)]
pub struct MovementTargetHighlight {
    pub player: Entity,
    pub source_area: Entity,
    pub max_tokens: usize,
    pub is_attack: bool,
    pub is_city_attack: bool,
}

/// Visual arrow entity connecting source to target
#[derive(Component, Debug)]
pub struct MovementArrow {
    pub source_area: Entity,
    pub target_area: Entity,
}

/// Marker for the movement UI root container
#[derive(Component, Default)]
pub struct MovementUiRoot;

/// Marker for the movement controls panel (OK/Cancel/+/- buttons)
#[derive(Component, Default)]
pub struct MovementControlsPanel;

/// Marker for the token count display text
#[derive(Component, Default)]
pub struct TokenCountDisplay;

/// Marker for the source area name display text
#[derive(Component, Default)]
pub struct SourceAreaDisplay;

/// Marker for movement arrow gizmo data
#[derive(Component, Debug)]
pub struct MovementArrowGizmo {
    pub source_pos: Vec2,
    pub target_pos: Vec2,
    pub color: Color,
}

#[cfg(test)]
mod movement_selection_state_tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    /// `AvailableMoves` is re-inserted after every move, which used to reset
    /// the cursor to the top of the list -- so moving out of one area threw
    /// the player back to the first one and they had to page all the way
    /// down again.
    #[test]
    fn the_cursor_stays_put_when_the_selected_area_still_has_moves() {
        let mut state = MovementSelectionState::default();
        state.resync_sources(&[e(1), e(2), e(3)]);
        state.next_source();
        state.next_source();
        assert_eq!(state.current_source(), Some(e(3)));

        // A move was made; the same three areas can still move.
        state.resync_sources(&[e(1), e(2), e(3)]);

        assert_eq!(
            state.current_source(),
            Some(e(3)),
            "stayed where the player was"
        );
    }

    /// When the area the player was working on is finished, resume from the
    /// one that followed it -- not from the start of the list.
    #[test]
    fn a_finished_area_hands_over_to_the_next_one_not_the_first() {
        let mut state = MovementSelectionState::default();
        state.resync_sources(&[e(1), e(2), e(3), e(4)]);
        state.next_source();
        assert_eq!(state.current_source(), Some(e(2)));

        // e(2) is spent and drops out of the offered sources.
        state.resync_sources(&[e(1), e(3), e(4)]);

        assert_eq!(
            state.current_source(),
            Some(e(3)),
            "carries on past the finished area"
        );
    }

    /// Finishing the last area wraps to the first remaining one, since there
    /// is nothing after it.
    #[test]
    fn finishing_the_last_area_wraps_around() {
        let mut state = MovementSelectionState::default();
        state.resync_sources(&[e(1), e(2), e(3)]);
        state.prev_source();
        assert_eq!(state.current_source(), Some(e(3)));

        state.resync_sources(&[e(1), e(2)]);

        assert_eq!(state.current_source(), Some(e(1)));
    }

    /// The offered sources come out of a `HashMap`, so their order varies
    /// between recalculations. Areas that are still in play must keep the
    /// places they had, or the list reshuffles under the player's cursor.
    #[test]
    fn surviving_areas_keep_their_order_however_the_new_list_arrives() {
        let mut state = MovementSelectionState::default();
        state.resync_sources(&[e(1), e(2), e(3)]);

        // Same areas, arbitrary new order, plus a newly-reachable one.
        state.resync_sources(&[e(3), e(4), e(1), e(2)]);

        assert_eq!(
            state.source_areas,
            vec![e(1), e(2), e(3), e(4)],
            "previous order preserved, newcomers appended"
        );
    }

    /// A skipped area must not steal the cursor when the list is rebuilt.
    #[test]
    fn resync_lands_on_an_unskipped_area() {
        let mut state = MovementSelectionState::default();
        state.resync_sources(&[e(1), e(2), e(3)]);
        state.next_source(); // e(2)
        state.skip_current_source(); // skips e(2), advances to e(3)
        assert_eq!(state.current_source(), Some(e(3)));

        // e(3) finishes; the next in line is the skipped e(2), so step over it.
        state.resync_sources(&[e(1), e(2)]);

        assert_eq!(
            state.current_source(),
            Some(e(1)),
            "skipped areas are not offered again"
        );
    }

    /// A fresh phase starts on the first area, not the second.
    #[test]
    fn a_fresh_phase_starts_at_the_first_area() {
        let mut state = MovementSelectionState::default();
        state.resync_sources(&[e(1), e(2), e(3)]);
        assert_eq!(state.current_source(), Some(e(1)));
    }
}
