use bevy::prelude::*;

/// One city the human player could choose to reduce, mirroring the data
/// carried by `EliminateCityMove` so the UI doesn't need to re-query it.
#[derive(Debug, Clone, Copy)]
pub struct CitySupportChoice {
    pub area: Entity,
    pub city: Entity,
    pub tokens_gained: usize,
    pub tokens_needed: usize,
}

/// Resource tracking the current city-support (reduce-a-city) selection
/// state for the human player.
#[derive(Resource, Default, Debug)]
pub struct CitySupportSelectionState {
    /// The human player currently selecting
    pub player: Option<Entity>,
    /// All cities the player could reduce to restore support
    pub choices: Vec<CitySupportChoice>,
    /// Index into `choices` for the currently focused city
    pub current_index: usize,
}

impl CitySupportSelectionState {
    pub fn clear(&mut self) {
        self.player = None;
        self.choices.clear();
        self.current_index = 0;
    }

    pub fn current_choice(&self) -> Option<CitySupportChoice> {
        self.choices.get(self.current_index).copied()
    }

    pub fn next_site(&mut self) {
        if !self.choices.is_empty() {
            self.current_index = (self.current_index + 1) % self.choices.len();
        }
    }

    pub fn prev_site(&mut self) {
        if !self.choices.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.choices.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }
}

/// Marker for the city support UI root container
#[derive(Component, Default)]
pub struct CitySupportUiRoot;

/// Marker for the currently focused city name/info display text
#[derive(Component, Default)]
pub struct CitySupportSiteDisplay;

/// Button actions for the city support UI
#[derive(Component, Debug, Clone)]
pub enum CitySupportButtonAction {
    EliminateCity,
    PrevSite,
    NextSite,
}
