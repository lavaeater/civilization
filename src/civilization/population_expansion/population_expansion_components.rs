use bevy::prelude::{Component, Entity, Reflect};
use bevy::utils::HashSet;

#[derive(Component, Debug)]
pub struct ExpandManually {
    pub required_tokens: usize,
}

impl ExpandManually {
    pub fn new(required_tokens: usize) -> Self {
        ExpandManually {
            required_tokens
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct ExpandAutomatically;

#[derive(Component, Debug)]
pub struct NeedsExpansion {
    pub areas_that_need_expansion: HashSet<Entity>,
}

impl NeedsExpansion {
    pub fn new(areas_that_need_expansion: HashSet<Entity>) -> Self {
        NeedsExpansion {
            areas_that_need_expansion
        }
    }
    
    pub fn remove(&mut self, area: Entity) {
        self.areas_that_need_expansion.remove(&area);
    }
}

#[derive(Component, Debug)]
pub struct AreaIsExpanding {
    pub players_that_must_expand: HashSet<Entity>,
}

impl AreaIsExpanding {
    pub fn new(players_that_must_expand: HashSet<Entity>) -> Self {
        AreaIsExpanding {
            players_that_must_expand
        }
    }
    
    pub fn remove(&mut self, player: Entity) {
        self.players_that_must_expand.remove(&player);
    }
    
    pub fn expansion_is_done(&self) -> bool {
        self.players_that_must_expand.is_empty()
    }
}