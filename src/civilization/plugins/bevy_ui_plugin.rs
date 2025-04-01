//! This example illustrates the various features of Bevy UI.

use crate::civilization::components::prelude::{TradeCounterType, TradeMoveType};
use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::events::HumanPlayerTradeCardsUpdated;
use crate::civilization::ui::ui_builder::UIBuilder;
use crate::stupid_ai::prelude::IsHuman;
use crate::GameActivity;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy::reflect::Enum;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
    prelude::*,
};
use itertools::Itertools;

const BG_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.25);
const CARD_COLOR: Color = Color::srgba(0.7, 0.6, 0.2, 0.8);

pub struct BevyUiPlugin;

impl Plugin for BevyUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(WinitSettings::desktop_app())
            .add_plugins(DebugUiPlugin)
            .add_systems(Update, toggle_overlay)
            .add_systems(OnEnter(GameActivity::StartGame), setup)
            .add_systems(Update, update_scroll_position)

            .add_systems(Update, handle_player_draws_cards);
    }
}

fn handle_player_draws_cards(
    mut reader: EventReader<HumanPlayerTradeCardsUpdated>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    trade_card_list: Query<Entity, With<TradeCardList>>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let mut new_commands = commands;
    for event in reader.read() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        debug!("Received Event!");
        if let Ok(trade_card_list) = trade_card_list.get_single() {
            debug!("Trade Card List exists!");
            if let Ok(player_trade_cards) = player_trade_cards.get(event.player_entity) {
                debug!("Player Trade Cards: {:?}", player_trade_cards);
                let grouped_cards = player_trade_cards.trade_cards_grouped_by_value_and_type();

                // Just debug values for now without rendering
                let mut ui_builder =
                    UIBuilder::start_from_entity(new_commands, trade_card_list, true);
                ui_builder.with_children(|mut b| {
                    for (value, group) in grouped_cards.iter().sorted_by_key(|(value, _)| *value) {
                        b.add_text_child(
                            format!("Cards with value: {}", value),
                            font.clone(),
                            24.0,
                            Some(Color::WHITE),
                        );
                        for (card_type, cards) in group.iter() {
                            b = b
                                .move_to_new_child()
                                .as_block(Val::Percent(100.), Val::Px(80.), CARD_COLOR)
                                .with_padding(UiRect::all(Val::Px(10.0)))
                                .move_to_new_child()
                                .with_size(Val::Percent(100.0), Val::Percent(100.0))
                                .with_text(
                                    format!("{}: {}", card_type, cards.len()),
                                    font.clone(),
                                    24.0,
                                    Some(Color::WHITE),
                                )
                                .parent()
                                .parent();
                        }
                    }
                });
                new_commands = ui_builder.build().1;
            }
        }
    }
    commands = new_commands;
}

#[derive(Component, Default)]
pub struct TradeCardUiRoot;

#[derive(Component, Default)]
pub struct TradeCardList;

fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    // root node
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mut root_ui = UIBuilder::new(commands);
    let grouped_cards = player_trade_cards
        .get_single()
        .unwrap()
        .trade_cards_grouped_by_value_and_type();

    root_ui
        .with_component::<TradeCardUiRoot>()
        .as_flex_col_with_props(Val::Percent(25.), Val::Percent(100.), BG_COLOR)
        .add_text_child("Your trade cards!", font.clone(), 24.0, Some(Color::WHITE))
        .move_to_new_child()
        .as_flex_col_with_props(Val::Percent(100.), Val::Percent(100.), BG_COLOR)
        .with_component::<TradeCardList>();

    // Get the built entity and commands back
    let (root_entity, commands) = root_ui.build();
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
