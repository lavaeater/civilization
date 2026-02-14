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
    
    pub fn current_source(&self) -> Option<Entity> {
        self.source_areas.get(self.current_source_index).copied()
    }
    
    pub fn is_current_skipped(&self) -> bool {
        self.current_source().is_some_and(|s| self.skipped_sources.contains(&s))
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
            && self.source_areas.iter().all(|s| self.skipped_sources.contains(s))
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
                self.current_source_index = (self.current_source_index + 1) % self.source_areas.len();
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
    
    pub fn select_target(&mut self, player: Entity, source: Entity, target: Entity, max_tokens: usize, is_attack: bool, is_city_attack: bool) {
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

/// Button actions for movement UI
#[derive(Component, Debug, Clone)]
pub enum MovementButtonAction {
    IncrementTokens,
    DecrementTokens,
    ConfirmMove,
    CancelMove,
    EndMovement,
    PrevSource,
    NextSource,
    SkipSource,
    SelectTarget { source: Entity, target: Entity, max_tokens: usize, is_attack: bool, is_city_attack: bool },
}

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
