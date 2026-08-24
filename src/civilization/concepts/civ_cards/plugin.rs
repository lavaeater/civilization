use crate::civilization::concepts::civ_cards::assets_resources::AvailableCivCards;
use crate::civilization::{
    BackToCardSelection, CivCardSelectionState, CivTradeUi, ConfirmCivCardPurchase, PaymentState,
    PlayerDoneAcquiringCivilizationCards, ProceedToPayment, RefreshCivCardsUi,
    ToggleCivCardSelection, begin_acquire_civ_cards, ensure_human_civ_cards_ui,
    handle_back_to_selection, handle_payment_adjust, handle_proceed_to_payment_message,
    handle_toggle_card_selection, handle_treasury_adjust, init_civ_cards, load_civ_cards,
    on_add_player_acquiring_civilization_cards, player_is_done, process_civ_card_purchase,
    refresh_civ_cards_ui, shuffle_trade_card_piles_on_exit,
};
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

/// Safety net: if the phase is left with a `CivTradeUi` still on screen (e.g.
/// a stuck/aborted purchase), despawn it so it can't block the next
/// `on_add_player_acquiring_civilization_cards` build (which refuses to run
/// while one already exists).
fn despawn_leftover_civ_trade_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<CivTradeUi>>,
) {
    for entity in &ui_query {
        commands.entity(entity).despawn();
    }
}

impl Plugin for CivCardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<AvailableCivCards>::new(&["cards.ron"]))
            .init_resource::<CivCardsAcquisition>()
            .init_resource::<CivCardSelectionState>()
            .init_resource::<PaymentState>()
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
                (
                    shuffle_trade_card_piles_on_exit,
                    despawn_leftover_civ_trade_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_toggle_card_selection,
                    handle_proceed_to_payment_message,
                    handle_back_to_selection,
                    handle_payment_adjust,
                    handle_treasury_adjust,
                    process_civ_card_purchase,
                    refresh_civ_cards_ui,
                    player_is_done,
                    ensure_human_civ_cards_ui,
                )
                    .run_if(in_state(GameActivity::AcquireCivilizationCards)),
            );
    }
}
