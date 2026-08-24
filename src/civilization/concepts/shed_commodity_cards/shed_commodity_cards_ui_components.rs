use crate::civilization::concepts::acquire_trade_cards::TradeCard;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Component, Entity, Resource};

/// Drives the human commodity-shed selection UI.
#[derive(Resource, Default, Debug)]
pub struct CommodityShedSelectionState {
    /// The human player currently choosing what to discard.
    pub player: Option<Entity>,
    pub must_discard: usize,
    /// Snapshot of the player's commodity holdings taken when the panel
    /// opened, so rows stay stable (and `held()` lookups are meaningful)
    /// while the player is deciding.
    pub holdings: Vec<(TradeCard, usize)>,
    /// How many of each card the player has chosen to discard so far.
    pub selected: HashMap<TradeCard, usize>,
}

impl CommodityShedSelectionState {
    pub fn populate(&mut self, player: Entity, must_discard: usize, holdings: Vec<(TradeCard, usize)>) {
        self.player = Some(player);
        self.must_discard = must_discard;
        self.holdings = holdings;
        self.selected.clear();
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn total_selected(&self) -> usize {
        self.selected.values().sum()
    }

    pub fn is_complete(&self) -> bool {
        self.must_discard > 0 && self.total_selected() == self.must_discard
    }

    pub fn held(&self, card: TradeCard) -> usize {
        self.holdings
            .iter()
            .find(|(c, _)| *c == card)
            .map_or(0, |(_, n)| *n)
    }

    pub fn selected_count(&self, card: TradeCard) -> usize {
        self.selected.get(&card).copied().unwrap_or(0)
    }

    /// Select one more copy of `card` to discard, if the player still holds
    /// an unselected copy and the total hasn't reached `must_discard` yet.
    pub fn increment(&mut self, card: TradeCard) {
        if self.total_selected() >= self.must_discard {
            return;
        }
        if self.selected_count(card) >= self.held(card) {
            return;
        }
        *self.selected.entry(card).or_insert(0) += 1;
    }

    pub fn decrement(&mut self, card: TradeCard) {
        if let Some(count) = self.selected.get_mut(&card) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                self.selected.remove(&card);
            }
        }
    }

    /// Drain and return the chosen discards, clearing state.
    pub fn take_result(&mut self) -> HashMap<TradeCard, usize> {
        let result = std::mem::take(&mut self.selected);
        self.clear();
        result
    }
}

#[derive(Component, Default)]
pub struct CommodityShedUiRoot;

#[derive(Component, Default)]
pub struct CommodityShedProgressText;

#[derive(Component)]
pub struct CommodityShedConfirmButton;

/// Tags the per-card discard-count label in a shed row so the update system
/// can find and refresh it.
#[derive(Component)]
pub struct CommodityShedCountText(pub TradeCard);
