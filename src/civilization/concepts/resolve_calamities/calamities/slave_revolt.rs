use bevy::prelude::{Entity, Reflect};

/// State for resolving Slave Revolt (rule 30.42).
///
/// Default: 15 tokens belonging to the primary victim may not support cities (30.421).
/// Modifiers (30.423):
/// - Mining       : +5 tokens cannot support
/// - Enlightenment: −5 tokens cannot support
/// - Both         : effects cancel (net 0 modifier)
///
/// Implementation note: since city support requires querying on-board token counts we
/// approximate by converting the affected-token count to a city count. One city
/// requires 5 supporting tokens, so 15 tokens ≈ 3 cities. The advance system
/// computes the actual city count from the player's board state.
#[derive(Debug, Clone, Default, Reflect)]
pub struct SlaveRevoltState {
    pub phase: SlaveRevoltPhase,
    /// Number of tokens that may not support cities (base 15 ± modifiers).
    pub tokens_cannot_support: usize,
    /// Derived from `tokens_cannot_support` at ComputeEffects time.
    pub cities_to_reduce: usize,
    pub selected_cities: Vec<Entity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub enum SlaveRevoltPhase {
    #[default]
    ComputeEffects,
    SelectCities,
    ApplyEffects,
    Complete,
}

impl SlaveRevoltState {
    pub fn new() -> Self {
        Self {
            tokens_cannot_support: 15,
            ..Default::default()
        }
    }

    /// Mining: +5 tokens cannot support cities (30.423).
    pub fn with_mining(mut self) -> Self {
        self.tokens_cannot_support += 5;
        self
    }

    /// Enlightenment: −5 tokens cannot support cities (30.423).
    pub fn with_enlightenment(mut self) -> Self {
        self.tokens_cannot_support = self.tokens_cannot_support.saturating_sub(5);
        self
    }

    /// Both Mining and Enlightenment: effects cancel — call neither modifier (30.423).
    /// Convenience constructor for the cancellation case.
    pub fn with_mining_and_enlightenment(self) -> Self {
        // Effects cancel; no change to base 15.
        self
    }

    /// Compute the number of cities to reduce from the actual board state.
    /// `on_board_tokens` is the number of tokens the player currently has on the map.
    /// Each city needs 5 supporting tokens; we find how many cities lose support.
    pub fn compute_cities_to_reduce(&mut self, on_board_tokens: usize, city_count: usize) {
        let affected = self.tokens_cannot_support.min(on_board_tokens);
        // Each unsupported group of 5 tokens forces one city to reduce.
        self.cities_to_reduce = (affected / 5).min(city_count);
    }

    pub fn select_city(&mut self, city_area: Entity) {
        if self.selected_cities.len() < self.cities_to_reduce {
            self.selected_cities.push(city_area);
        }
    }

    pub fn selection_complete(&self) -> bool {
        self.selected_cities.len() >= self.cities_to_reduce || self.cities_to_reduce == 0
    }
}
