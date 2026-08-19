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

#[cfg(test)]
mod tests {
    use super::*;

    /// Rule 30.421: 15 tokens belonging to the primary victim may not support cities.
    #[test]
    fn base_is_15_tokens() {
        assert_eq!(SlaveRevoltState::new().tokens_cannot_support, 15);
    }

    /// Rule 30.423: Mining adds 5 tokens that cannot support.
    #[test]
    fn mining_adds_5_tokens() {
        assert_eq!(SlaveRevoltState::new().with_mining().tokens_cannot_support, 20);
    }

    /// Rule 30.423: Enlightenment removes 5 tokens from the unsupported count.
    #[test]
    fn enlightenment_removes_5_tokens() {
        assert_eq!(SlaveRevoltState::new().with_enlightenment().tokens_cannot_support, 10);
    }

    /// Rule 30.423: holding both Mining and Enlightenment cancels the modifiers — net base 15.
    #[test]
    fn mining_and_enlightenment_cancel_out() {
        assert_eq!(
            SlaveRevoltState::new().with_mining_and_enlightenment().tokens_cannot_support,
            15
        );
    }

    /// One city needs 5 supporting tokens to stay up; the affected-token count is
    /// converted to a city count, capped by however many cities the player actually has.
    #[test]
    fn cities_to_reduce_derived_from_affected_tokens_and_board_state() {
        let mut state = SlaveRevoltState::new(); // 15 tokens
        state.compute_cities_to_reduce(15, 10);
        assert_eq!(state.cities_to_reduce, 3); // 15 / 5

        let mut capped = SlaveRevoltState::new();
        capped.compute_cities_to_reduce(15, 2); // only 2 cities exist
        assert_eq!(capped.cities_to_reduce, 2);

        let mut fewer_tokens = SlaveRevoltState::new();
        fewer_tokens.compute_cities_to_reduce(7, 10); // fewer than 15 on board
        assert_eq!(fewer_tokens.cities_to_reduce, 1); // 7 / 5 = 1
    }
}
