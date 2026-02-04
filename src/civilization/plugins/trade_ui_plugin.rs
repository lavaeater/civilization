//! This example illustrates the various features of Bevy UI.

use crate::civilization::concepts::acquire_trade_cards::trade_card_components::PlayerTradeCards;
use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::{
    TradeCard, TradeCardTrait,
};
use crate::civilization::concepts::acquire_trade_cards::trade_card_events::HumanPlayerTradeCardsUpdated;
use crate::civilization::concepts::trade::trade_components::{TradeCardList, TradeCardUiRoot};
use crate::civilization::ui::ui_builder::{
    ButtonPartial, NodePartial, UIBuilder, UiBuilderDefaults, BG_COLOR, BORDER_COLOR, CARD_COLOR,
    TEXT_COLOR,
};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::prelude::*;
use crate::civilization::sandbox_plugin::{build_trade_card_list, handle_trade_scroll_input, setup_trade_ui};

pub struct TradeUiPlugin;

impl Plugin for TradeUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(WinitSettings::desktop_app())
            .insert_resource(UiBuilderDefaults::new())
            .add_systems(OnEnter(GameActivity::StartGame), setup_trade_ui)
            .add_systems(Update, (handle_player_draws_cards, handle_trade_scroll_input));
    }
}

fn commodity_card_max_set_size(card: TradeCard) -> usize {
    match card.value() {
        1..=4 => 8,
        5 => 7,
        6 => 6,
        7 => 5,
        8 | 9 => 4,
        _ => 8,
    }
}

fn add_commodity_card(
    b: &mut UIBuilder,
    card_type: TradeCard,
    count: usize,
    defaults: &UiBuilderDefaults,
) {
    let active_index = count.clamp(1, commodity_card_max_set_size(card_type));
    let max_set_size = commodity_card_max_set_size(card_type);
    let highlight_bg = Color::srgba(1.0, 1.0, 1.0, 0.25);
    let normal_bg = Color::srgba(0.0, 0.0, 0.0, 0.0);
    let highlight_border = Color::srgba(1.0, 1.0, 1.0, 0.85);

    b.as_block_px(160., 240., CARD_COLOR)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .border_all_px(2.0, BORDER_COLOR)
        .padding_all_px(4.0)
        .with_child(|b| {
            // Top set-value boxes (first half)
            b.display(Display::Flex)
                .flex_direction_row()
                .flex_wrap()
                // .justify_content(JustifyContent::SpaceBetween)
                // .align_items(AlignItems::Center)
                .row_gap_px(2.0)
                .column_gap_px(2.0)
                .width_percent(100.)
                .height_px(40.0)
                .foreach_child(1..=max_set_size.min(4), |b, n| {
                    let is_active = n == active_index;
                    let bg = if is_active { highlight_bg } else { normal_bg };
                    let bc = if is_active {
                        highlight_border
                    } else {
                        BORDER_COLOR
                    };
                    b.as_block_px(18., 18., bg)
                        .border_all_px(1., bc)
                        // .align_content(AlignContent::Center)
                        // .align_items(AlignItems::Center)
                        // .justify_content(JustifyContent::Center)
                        // .justify_items(JustifyItems::Center)
                        .with_text(
                            format!("{}", (n * n) * card_type.value()),
                            Some(defaults.base_font.clone()),
                            Some(10.0),
                            Some(TEXT_COLOR),
                            None, None
                        );
                });
        });

    // Center value + name
    //     b = b
    //         .child()
    //         .with(
    //             NodePartial::new()
    //                 .display(Display::Flex)
    //                 .flex_direction(FlexDirection::Column)
    //                 .justify_content(JustifyContent::Center)
    //                 .align_items(AlignItems::Center)
    //                 .width(Val::Percent(100.))
    //                 .flex_grow(1.0),
    //         )
    //         .with_children(|mut b| {
    //             b = b
    //                 .add_text_child(
    //                     format!("{}", card_type.value()),
    //                     Some(defaults.base_font.clone()),
    //                     Some(64.0),
    //                     Some(Color::srgba(1.0, 1.0, 1.0, 0.9)),
    //                 )
    //                 .parent();
    //             b = b
    //                 .add_text_child(
    //                     format!("{}", card_type),
    //                     Some(defaults.base_font.clone()),
    //                     Some(18.0),
    //                     Some(Color::srgba(0.1, 0.1, 0.1, 0.95)),
    //                 )
    //                 .parent();
    //         })
    //         .parent();
    //
    //     // Bottom set-value boxes (second half)
    //     if max_set_size > 4 {
    //         _ = b
    //             .child()
    //             .with(
    //                 NodePartial::new()
    //                     .display(Display::Flex)
    //                     .flex_direction(FlexDirection::Row)
    //                     .flex_wrap(FlexWrap::Wrap)
    //                     .justify_content(JustifyContent::SpaceBetween)
    //                     .align_items(AlignItems::Center)
    //                     .row_gap(Val::Px(2.0))
    //                     .column_gap(Val::Px(2.0))
    //                     .width(Val::Percent(100.))
    //                     .height(Val::Px(40.0)),
    //             )
    //             .with_children(|mut b| {
    //                 for n in 5..=max_set_size {
    //                     let is_active = n == active_index;
    //                     let bg = if is_active { highlight_bg } else { normal_bg };
    //                     let bc = if is_active { highlight_border } else { BORDER_COLOR };
    //                     b = b
    //                         .child()
    //                         .as_block(Val::Px(36.), Val::Px(18.), bg)
    //                         .with_border(UiRect::all(Val::Px(1.0)), bc)
    //                         .with_text(
    //                             format!("{}", (n * n) * card_type.value()),
    //                             Some(defaults.base_font.clone()),
    //                             Some(12.0),
    //                             Some(TEXT_COLOR),
    //                         )
    //                         .parent();
    //                 }
    //             })
    //             .parent();
    //     }
    // })
    // .parent();
}

fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_defaults: ResMut<UiBuilderDefaults>,
    human_players: Query<Entity, With<IsHuman>>,
    mut pulled_card_event_writer: MessageWriter<HumanPlayerTradeCardsUpdated>,
) {
    // root node
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_defaults.base_font = font.clone();
    ui_defaults.bg_color = BG_COLOR;
    ui_defaults.text_color = TEXT_COLOR;
    ui_defaults.font_size = 18.0;
    ui_defaults.border_color = BORDER_COLOR;
    ui_defaults.button_def = Some(ButtonPartial {
        border_radius: Some(BorderRadius::MAX),
        ..default()
    });
    ui_defaults.node_def = Some(
        NodePartial::new()
            .border(UiRect::all(Val::Px(2.5)))
            .border_color(BORDER_COLOR)
            .border_radius(BorderRadius::ZERO)
            .padding(UiRect::all(Val::Px(5.0)))
            .margin(UiRect::all(Val::Px(2.5)))
            .justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .align_content(AlignContent::Center)
            .justify_items(JustifyItems::Center)
            .justify_self(JustifySelf::Auto),
    );

    let mut root_ui = UIBuilder::new(commands, Some(ui_defaults.clone()));

    root_ui
        .with_component::<TradeCardUiRoot>()
        .as_flex_col(Val::Percent(25.), Val::Percent(100.))
        .text_node("Your trade cards!")
        .child()
        .as_flex_col(Val::Percent(100.), Val::Percent(100.))
        .with_component::<TradeCardList>();

    let (_root_entity, _commands) = root_ui.build();
    for human_player in human_players.iter() {
        info!("Human player: {}", human_player);
        pulled_card_event_writer.write(HumanPlayerTradeCardsUpdated::new(human_player));
    }
}

fn handle_player_draws_cards(
    mut reader: MessageReader<HumanPlayerTradeCardsUpdated>,
    commands: Commands,
    ui_builder_defaults: Res<UiBuilderDefaults>,
    trade_card_list: Query<Entity, With<TradeCardList>>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let mut new_commands = commands;
    for event in reader.read() {
        info!("Player {} has updated cards", event.player_entity);
        if let Ok(trade_card_list_entity) = trade_card_list.single() {
            info!("Trade card list exists");
            if let Ok(player_trade_cards) = player_trade_cards.get(event.player_entity) {
                let mut ui_builder = UIBuilder::start_from_entity(
                    new_commands,
                    trade_card_list_entity,
                    true,
                    Some(ui_builder_defaults.clone()),
                );
                build_trade_card_list(&mut ui_builder, player_trade_cards);
                new_commands = ui_builder.build().1;
            }
        }
    }
}
