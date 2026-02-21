use bevy::prelude::{Entity, Message};
use crate::civilization::CivCardName;
use crate::civilization::concepts::acquire_trade_cards::TradeCard;
use bevy::platform::collections::HashMap;

#[derive(Message)]
pub struct PlayerDoneAcquiringCivilizationCards(pub Entity);

/// Message to toggle selection of a civilization card
#[derive(Message)]
pub struct ToggleCivCardSelection(pub CivCardName);

/// Message to proceed to payment phase
#[derive(Message)]
pub struct ProceedToPayment;

/// Message to go back to card selection phase
#[derive(Message)]
pub struct BackToCardSelection;

/// Message to confirm purchase with selected payment
#[derive(Message)]
pub struct ConfirmCivCardPurchase {
    pub player: Entity,
    pub cards_to_buy: Vec<CivCardName>,
    pub payment: HashMap<TradeCard, usize>,
}

/// Message to refresh the civ cards UI
#[derive(Message)]
pub struct RefreshCivCardsUi;