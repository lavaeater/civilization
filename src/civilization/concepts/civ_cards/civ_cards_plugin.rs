use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::platform::collections::HashSet;
use bevy::prelude::{Entity, Res};
use bevy::prelude::Resource;
use bevy::prelude::{
    in_state, Add, App, Commands, Component, Has, IntoScheduleConfigs, Message, MessageReader,
    NextState, On, OnEnter, Plugin, Query, ResMut, Update, With,
};
use lava_ui_builder::{LavaTheme, UIBuilder};

pub struct CivCardsPlugin;

#[derive(Message)]
pub struct PlayerDoneAcquiringCivilizationCards(Entity);

#[derive(Component)]
pub struct PlayerAcquiringCivilizationCards;

#[derive(Resource, Default)]
pub struct CivCardsAcquisition {
    pub players: HashSet<Entity>,
    pub human_players: HashSet<Entity>,
}

#[derive(Component, Default)]
struct CivTradeUi;

impl CivCardsAcquisition {
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

impl Plugin for CivCardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CivCardsAcquisition>()
            .add_observer(on_add_player_acquiring_civilization_cards)
            .add_systems(
                OnEnter(GameActivity::AcquireCivilizationCards),
                (begin_acquire_civ_cards),
            )
            .add_systems(
                Update,
                (player_is_done).run_if(in_state(GameActivity::AcquireCivilizationCards)),
            );
    }
}

fn on_add_player_acquiring_civilization_cards(
    trigger: On<Add, PlayerAcquiringCivilizationCards>,
    is_human_player: Query<(&Player, &IsHuman)>,
    ui_exists_query: Query<(), With<CivTradeUi>>,
    commands: Commands,
    theme: Res<LavaTheme>,
    cards: Res<V>
) {
    if is_human_player.contains(trigger.entity) && ui_exists_query.is_empty() {
        let mut builder = UIBuilder::new(commands, Some(theme.clone()));
        builder.component::<CivTradeUi>();
    }
}

fn player_is_done(
    mut commands: Commands,
    mut done_reader: MessageReader<PlayerDoneAcquiringCivilizationCards>,
    mut civ_cards_acquisition: ResMut<CivCardsAcquisition>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for done in done_reader.read() {
        commands
            .entity(done.0)
            .remove::<PlayerAcquiringCivilizationCards>();
        civ_cards_acquisition.players.remove(&done.0);
        civ_cards_acquisition.human_players.remove(&done.0);
    }
    if civ_cards_acquisition.is_empty() {
        next_state.set(GameActivity::PopulationExpansion)
    }
}

fn begin_acquire_civ_cards(
    mut commands: Commands,
    mut civ_cards_acquisition: ResMut<CivCardsAcquisition>,
    players: Query<(Entity, Has<IsHuman>), With<Player>>,
) {
    /*
    We never check - we simply create a resource or something that contains all players,
    and as players send the done-message, we remove them from the list. And when the list is
    empty, we move to the next activity.
    */
    for ((entity, is_human)) in players.iter() {
        commands
            .entity(entity)
            .insert(PlayerAcquiringCivilizationCards);
        civ_cards_acquisition.players.insert(entity);
        if is_human {
            civ_cards_acquisition.human_players.insert(entity);
        }
    }
}
