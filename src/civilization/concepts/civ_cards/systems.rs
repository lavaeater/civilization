use crate::civilization::{AvailableCivCards, CardHandle, CivCardDefinition, CivCardType, CivCardsAcquisition, CivTradeUi, Credits, PlayerAcquiringCivilizationCards, PlayerDoneAcquiringCivilizationCards};
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::prelude::{percent, px, Add, Commands, Entity, Has, MessageReader, NextState, On, Query, Res, ResMut, Val, With};
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
        let mut theme_to_use = theme.clone();
        theme_to_use.text.label_size = 12.0;
        let mut builder = UIBuilder::new(commands, Some(theme_to_use));
        builder.component::<CivTradeUi>().add_panel(|panel| {
            let panel_color = Color::srgba(0.1, 0.1, 0.1, 0.9);
            panel
                .display_flex()
                .flex_row()
                .size(percent(95.), percent(90.))
                .bg_color(panel_color)
                .padding_all_px(16.0)
                .gap_px(8.0)
                .overflow_scroll_y();

            panel.foreach_child(CivCardType::all_types().iter(), |col_builder, card_type| {
                let col_color = Color::srgba(0.15, 0.15, 0.15, 0.8);
                col_builder
                    .display_flex()
                    .flex_column()
                    .width(px(220.0))
                    .height(percent(100.))
                    .bg_color(col_color)
                    .padding_all_px(8.0)
                    .row_gap_px(8.0)
                    .overflow_scroll_y()
                    .border_radius_all_px(4.0);

                col_builder.with_child(|header| {
                    header
                        .display_flex()
                        .justify_center()
                        .padding_all_px(4.0)
                        .margin_btm(Val::Px(8.0))
                        .default_text(card_type.to_string());
                });

                col_builder.foreach_child(&cards.get_cards(card_type), |card_builder, card| {
                    create_civ_card_panel(card_builder, card);
                });
            });
        });
    }
}

fn create_civ_card_panel(card_builder: &mut UIBuilder, card: &CivCardDefinition) {
    let card_bg = Color::srgba(0.2, 0.2, 0.25, 1.0);
    card_builder
        .display_flex()
        .flex_column()
        .bg_color(card_bg)
        .padding_all_px(8.0)
        .border_radius_all_px(4.0)
        .row_gap_px(4.0);

    card_builder.with_child(|name_row| {
        name_row
            .display_flex()
            .flex_row()
            .justify_space_between()
            .with_child(|name| {
                name.default_text(card.name.to_string());
            })
            .with_child(|cost| {
                cost.default_text(format!("Cost: {}", card.cost));
            });
    });

    card_builder.with_child(|desc| {
        desc.default_text(&card.description);
    });

    if !card.credits.is_empty() {
        card_builder.with_child(|credits_section| {
            credits_section
                .display_flex()
                .flex_column()
                .row_gap_px(2.0);

            credits_section.with_child(|label| {
                label.default_text("Credits:");
            });

            credits_section.foreach_child(card.credits.iter(), |credit_row, credit| {
                credit_row.default_text(format_credit(credit));
            });
        });
    }
}

fn format_credit(credit: &Credits) -> String {
    match credit {
        Credits::ToType(card_type, amount) => format!("  +{} to {:?}", amount, card_type),
        Credits::ToAll(amount) => format!("  +{} to all", amount),
        Credits::ToSpecificCard(card_name, amount) => format!("  +{} to {}", amount, card_name),
    }
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
