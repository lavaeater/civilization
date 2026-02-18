use crate::civilization::{begin_acquire_civ_cards, init_civ_cards, load_civ_cards, on_add_player_acquiring_civilization_cards, player_is_done};
use crate::{GameActivity, GameState};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use crate::civilization::concepts::civ_cards::assets_resources::AvailableCivCards;

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
            .add_observer(on_add_player_acquiring_civilization_cards)
            .add_systems(OnEnter(GameState::Loading), load_civ_cards)
            .add_systems(OnEnter(GameState::Playing), init_civ_cards)
            .add_systems(
                OnEnter(GameActivity::AcquireCivilizationCards),
                (init_civ_cards, begin_acquire_civ_cards).chain(),
            )
            .add_systems(
                Update,
                (player_is_done).run_if(in_state(GameActivity::AcquireCivilizationCards)),
            );
    }
}

