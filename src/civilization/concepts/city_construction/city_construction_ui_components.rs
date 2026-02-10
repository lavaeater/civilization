use bevy::prelude::*;

/// Resource tracking the current city construction selection state for human player
#[derive(Resource, Default, Debug)]
pub struct CityConstructionSelectionState {
    /// The human player currently selecting
    pub player: Option<Entity>,
    /// All areas where the player can build a city
    pub build_sites: Vec<Entity>,
    /// Index into build_sites for the currently focused site
    pub current_site_index: usize,
}

impl CityConstructionSelectionState {
    pub fn clear(&mut self) {
        self.player = None;
        self.build_sites.clear();
        self.current_site_index = 0;
    }

    pub fn current_site(&self) -> Option<Entity> {
        self.build_sites.get(self.current_site_index).copied()
    }

    pub fn next_site(&mut self) {
        if !self.build_sites.is_empty() {
            self.current_site_index = (self.current_site_index + 1) % self.build_sites.len();
        }
    }

    pub fn prev_site(&mut self) {
        if !self.build_sites.is_empty() {
            if self.current_site_index == 0 {
                self.current_site_index = self.build_sites.len() - 1;
            } else {
                self.current_site_index -= 1;
            }
        }
    }
}

/// Marker for the city construction UI root container
#[derive(Component, Default)]
pub struct CityConstructionUiRoot;

/// Marker for the build site name display text
#[derive(Component, Default)]
pub struct BuildSiteDisplay;

/// Button actions for city construction UI
#[derive(Component, Debug, Clone)]
pub enum CityConstructionButtonAction {
    BuildCity,
    SkipCity,
    PrevSite,
    NextSite,
}
