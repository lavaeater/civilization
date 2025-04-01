//! This example illustrates the various features of Bevy UI.

use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::events::HumanPlayerTradeCardsUpdated;
use crate::civilization::ui::ui_builder::{
    ButtonPartial, NodePartial, UIBuilder, UiBuilderDefaults, BG_COLOR, BORDER_COLOR, CARD_COLOR,
    TEXT_COLOR,
};
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
    prelude::*,
};
use itertools::Itertools;

pub struct BevyUiPlugin;

impl Plugin for BevyUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(WinitSettings::desktop_app())
            .add_plugins(DebugUiPlugin)
            .insert_resource(UiBuilderDefaults::new())
            .add_systems(Update, toggle_overlay)
            .add_systems(OnEnter(GameActivity::StartGame), setup)
            .add_systems(Update, update_scroll_position)
            .add_systems(Update, handle_player_draws_cards);
    }
}

fn handle_player_draws_cards(
    mut reader: EventReader<HumanPlayerTradeCardsUpdated>,
    commands: Commands,
    ui_builder_defaults: Res<UiBuilderDefaults>,
    trade_card_list: Query<Entity, With<TradeCardList>>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let mut new_commands = commands;
    for event in reader.read() {
        if let Ok(trade_card_list) = trade_card_list.get_single() {
            if let Ok(player_trade_cards) = player_trade_cards.get(event.player_entity) {
                let grouped_cards = player_trade_cards.trade_cards_grouped_by_value_and_type();
                let mut ui_builder = UIBuilder::start_from_entity(
                    new_commands,
                    trade_card_list,
                    true,
                    Some(ui_builder_defaults.clone()),
                );
                ui_builder.with_children(|mut b| {
                    for (value, group) in grouped_cards.iter().sorted_by_key(|(value, _)| *value) {
                        b.add_default_text_child(format!("Cards with value: {}", value));
                        for (card_type, cards) in group.iter() {
                            b
                                .child()
                                .as_block(Val::Percent(100.), Val::Px(80.), CARD_COLOR)
                                .with_padding(UiRect::all(Val::Px(10.0)))
                                .child()
                                .with_size(Val::Percent(100.0), Val::Percent(100.0))
                                .with_default_text(format!("{}: {}", card_type, cards.len()))
                                .parent();
                        }
                    }
                });
                new_commands = ui_builder.build().1;
            }
        }
    }
}

#[derive(Component, Default)]
pub struct TradeCardUiRoot;

#[derive(Component, Default)]
pub struct TradeCardList;

fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_defaults: ResMut<UiBuilderDefaults>,
) {
    // root node
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_defaults.base_font = font.clone();
    ui_defaults.bg_color = BG_COLOR;
    ui_defaults.text_color = TEXT_COLOR;
    ui_defaults.font_size = 24.0;
    ui_defaults.border_color = BORDER_COLOR;
    ui_defaults.button_def = Some(ButtonPartial {
        border_radius: Some(BorderRadius::MAX),
        ..default()
    });

    ui_defaults.node_def = Some(NodePartial {
        display: Some(Display::Flex),
        border_radius: Some(BorderRadius::ZERO),
        margin: Some(UiRect::all(Val::Px(10.0))),
        ..default()
    });

    let mut root_ui = UIBuilder::new(commands, Some(ui_defaults.clone()));

    root_ui
        .with_component::<TradeCardUiRoot>()
        .as_flex_col(Val::Percent(25.), Val::Percent(100.))
        .add_default_text_child("Your trade cards!")
        .child()
        .as_flex_col(Val::Percent(100.), Val::Percent(100.))
        .with_component::<TradeCardList>();

    // Get the built entity and commands back
    let (_root_entity, _commands) = root_ui.build();
}

fn toggle_overlay(mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>) {
    info_once!("Will enable overlays automatically perhaps");
    if options.enabled {
        options.toggle();
    }
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 20., mouse_wheel_event.y * 20.),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}
