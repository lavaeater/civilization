use crate::civilization::{AvailableCivCards, CardHandle, CivCardDefinition, CivCardType, CivCardsAcquisition, CivTradeUi, PlayerAcquiringCivilizationCards, PlayerDoneAcquiringCivilizationCards};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::prelude::{percent, px, Add, Commands, Entity, Has, MessageReader, NextState, On, Query, Res, ResMut, With};
use lava_ui_builder::{LavaTheme, UIBuilder};

pub fn load_civ_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    //Load the cards here as well, while we're at it
    let card_handle = asset_server.load("definitions/civilization.cards.ron");
    let h = CardHandle(card_handle);
    commands.insert_resource(h);
}

pub fn init_civ_cards(
    mut commands: Commands,
    card_handle: Res<CardHandle>,
    maps: Res<Assets<AvailableCivCards>>,
) {
    if let Some(cards) = maps.get(card_handle.0.id()) {
        commands.insert_resource(AvailableCivCards {
            cards: cards.cards.clone(),
        });
    }
}

pub fn on_add_player_acquiring_civilization_cards(
    trigger: On<Add, PlayerAcquiringCivilizationCards>,
    is_human_player: Query<(&Player, &IsHuman)>,
    ui_exists_query: Query<(), With<CivTradeUi>>,
    commands: Commands,
    theme: Res<LavaTheme>,
    cards: Res<AvailableCivCards>,
) {
    if is_human_player.contains(trigger.entity) && ui_exists_query.is_empty() {
        let mut builder = UIBuilder::new(commands, Some(theme.clone()));
        builder.component::<CivTradeUi>().add_panel(|panel| {
            let panel_color = Color::srgba(0.1, 0.1, 0.1, 0.5);
            panel
                .display_flex()
                .flex_dir_column()
                .size(percent(80.), percent(80.))
                .bg_color(panel_color);
            
            panel.foreach_child(CivCardType::all_types().iter(), |card_type_builder, card_type| {
                card_type_builder
                    .height(px(250.0))
                    .add_text_child(card_type.to_string(), None, None,None)
                    .foreach_child(&cards.get_cards(card_type), |child, card| {
                        create_civ_card_panel(child, card)
                    });
            });
            
   
        });
    }
}

fn create_civ_card_panel(card_builder: &mut UIBuilder, card: &CivCardDefinition) {
    card_builder
        .flex_row()
        .add_text_child(card.name.to_string(), None, None, None);
}

pub fn player_is_done(
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

pub fn begin_acquire_civ_cards(
    mut commands: Commands,
    mut civ_cards_acquisition: ResMut<CivCardsAcquisition>,
    players: Query<(Entity, Has<IsHuman>), With<Player>>,
) {
    /*
    We never check - we simply create a resource or something that contains all players,
    and as players send the done-message, we remove them from the list. And when the list is
    empty, we move to the next activity.
    */
    for (entity, is_human) in players.iter() {
        commands
            .entity(entity)
            .insert(PlayerAcquiringCivilizationCards);
        civ_cards_acquisition.players.insert(entity);
        if is_human {
            civ_cards_acquisition.human_players.insert(entity);
        }
    }
}
