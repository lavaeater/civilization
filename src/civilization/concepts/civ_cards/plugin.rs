use crate::civilization::concepts::civ_cards::assets_resources::AvailableCivCards;
use crate::civilization::{begin_acquire_civ_cards, handle_back_button, handle_back_to_selection, handle_civ_card_clicks, handle_confirm_purchase_button, handle_done_button, handle_proceed_to_payment, handle_proceed_to_payment_message, handle_toggle_card_selection, init_civ_cards, load_civ_cards, on_add_player_acquiring_civilization_cards, player_is_done, process_civ_card_purchase, refresh_civ_cards_ui, shuffle_trade_card_piles_on_exit, BackToCardSelection, CivCardSelectionState, ConfirmCivCardPurchase, PlayerDoneAcquiringCivilizationCards, ProceedToPayment, RefreshCivCardsUi, ToggleCivCardSelection};
use crate::{GameActivity, GameState};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

pub struct CivCardsPlugin;

#[derive(Resource, Default)]
pub struct CivCardsAcquisition {
    pub players: HashSet<Entity>,
    pub human_players: HashSet<Entity>,
}

impl CivCardsAcquisition {
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

impl Plugin for CivCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RonAssetPlugin::<AvailableCivCards>::new(&["cards.ron"]))
            .init_resource::<CivCardsAcquisition>()
            .init_resource::<CivCardSelectionState>()
            .add_observer(on_add_player_acquiring_civilization_cards)
            .add_message::<PlayerDoneAcquiringCivilizationCards>()
            .add_message::<ToggleCivCardSelection>()
            .add_message::<ProceedToPayment>()
            .add_message::<BackToCardSelection>()
            .add_message::<ConfirmCivCardPurchase>()
            .add_message::<RefreshCivCardsUi>()
            .add_systems(OnEnter(GameState::Loading), load_civ_cards)
            .add_systems(OnEnter(GameState::Playing), init_civ_cards)
            .add_systems(
                OnEnter(GameActivity::AcquireCivilizationCards),
                (init_civ_cards, begin_acquire_civ_cards).chain(),
            )
            .add_systems(
                OnExit(GameActivity::AcquireCivilizationCards),
                shuffle_trade_card_piles_on_exit,
            )
            .add_systems(
                Update,
                (
                    handle_civ_card_clicks,
                    handle_toggle_card_selection,
                    handle_proceed_to_payment,
                    handle_proceed_to_payment_message,
                    handle_done_button,
                    handle_back_button,
                    handle_back_to_selection,
                    handle_confirm_purchase_button,
                    process_civ_card_purchase,
                    refresh_civ_cards_ui,
                    player_is_done,
                )
                    .run_if(in_state(GameActivity::AcquireCivilizationCards)),
            );
    }
}
