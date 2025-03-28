//! This example illustrates the various features of Bevy UI.

use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::events::HumanPlayerPulledTradeCard;
use crate::civilization::ui::ui_builder::UIBuilder;
use crate::stupid_ai::prelude::IsHuman;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy::{
    color::palettes::{basic::LIME, css::DARK_GRAY},
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
    prelude::*,
    ui::widget::NodeImageMode,
    winit::WinitSettings,
};
use std::f32::consts::PI;

pub struct BevyUiPlugin;

impl Plugin for BevyUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinitSettings::desktop_app())
            .add_plugins(DebugUiPlugin)
            .add_systems(Update, toggle_overlay)
            .add_systems(Startup, setup)
            .add_systems(Update, update_scroll_position)
            .add_systems(Update, handle_player_draws_cards);
    }
}

fn handle_player_draws_cards(
    mut reader: EventReader<HumanPlayerPulledTradeCard>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    trade_card_list: Query<Entity, With<TradeCardList>>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    for event in reader.read() {
        {
            if let Ok(trade_card_list) = trade_card_list.get_single() {
                if let Ok(player_trade_cards) = player_trade_cards.get_single() {
                    let grouped_cards = player_trade_cards.trade_cards_grouped_by_value_and_type();
                    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
                    let bg_color = Color::srgba(0.5, 0.5, 0.5, 0.25);

                    commands = UIBuilder::from_entity(commands, trade_card_list, true)
                        .block(100.0, 100.0, bg_color)
                        .build_command();
                }
            }
        }
    }
}

#[derive(Component, Default)]
pub struct TradeCardUiRoot;

#[derive(Component, Default)]
pub struct TradeCardList;

fn setup(commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let bg_color = Color::srgba(0.5, 0.5, 0.5, 0.25);
    let root_ui = UIBuilder::new(commands)
        .block_with::<TradeCardUiRoot>(25.0, 100.0, bg_color)
        .width(Val::Percent(100.0))
        .height(Val::Percent(100.0))
        .text("Your trade cards!", font.clone(), 24.0, Some(Color::WHITE))
        .parent()
        .block_with::<TradeCardList>(100., 100., bg_color)
        .build();
}

fn toggle_overlay(mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>) {
    info_once!("Will enable overlays automatically perhaps");
    if !options.enabled {
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
