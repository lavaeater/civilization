use crate::civilization::CivCardName;
use bevy::platform::collections::HashSet;
use bevy::prelude::{Component, Entity, Resource};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Serialize, Deserialize, Default)]
pub struct PlayerCivilizationCards {
    pub cards: HashSet<CivCardName>,
}

impl PlayerCivilizationCards {
    pub fn owns(&self, card: &CivCardName) -> bool {
        self.cards.contains(card)
    }
    
    pub fn has_prerequisites(&self, prerequisites: &[CivCardName]) -> bool {
        prerequisites.iter().all(|prereq| self.cards.contains(prereq))
    }
    
    pub fn add_card(&mut self, card: CivCardName) {
        self.cards.insert(card);
    }
}

#[derive(Component)]
pub struct PlayerAcquiringCivilizationCards;

#[derive(Component, Default)]
pub struct CivTradeUi;

/// Resource tracking the current card selection state for the human player
#[derive(Resource, Default)]
pub struct CivCardSelectionState {
    pub selected_cards: HashSet<CivCardName>,
    pub player_entity: Option<Entity>,
    pub phase: CivCardPurchasePhase,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CivCardPurchasePhase {
    #[default]
    SelectingCards,
    SelectingPayment,
}

impl CivCardSelectionState {
    pub fn toggle_card(&mut self, card: CivCardName) {
        if self.selected_cards.contains(&card) {
            self.selected_cards.remove(&card);
        } else {
            self.selected_cards.insert(card);
        }
    }
    
    pub fn is_selected(&self, card: &CivCardName) -> bool {
        self.selected_cards.contains(card)
    }
    
    pub fn clear(&mut self) {
        self.selected_cards.clear();
        self.phase = CivCardPurchasePhase::SelectingCards;
    }
    
    pub fn total_selected(&self) -> usize {
        self.selected_cards.len()
    }
}

/// Marker component for a civ card button in the UI
#[derive(Component)]
pub struct CivCardButton {
    pub card_name: CivCardName,
}

/// Marker for the selected cards summary panel
#[derive(Component, Default)]
pub struct SelectedCardsSummary;

/// Marker for the payment selection panel
#[derive(Component, Default)]
pub struct PaymentSelectionPanel;

/// Marker for the confirm purchase button
#[derive(Component, Default)]
pub struct ConfirmPurchaseButton;

/// Marker for the done button (skip purchasing)
#[derive(Component, Default)]
pub struct DonePurchasingButton;

/// Marker for proceed to payment button
#[derive(Component, Default)]
pub struct ProceedToPaymentButton;

/// Marker for back to selection button
#[derive(Component, Default)]
pub struct BackToSelectionButton;