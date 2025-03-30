//! This example illustrates the various features of Bevy UI.

use crate::civilization::concepts::trade_cards::components::PlayerTradeCards;
use crate::civilization::concepts::trade_cards::events::HumanPlayerTradeCardsUpdated;
use crate::civilization::ui::ui_builder::UIBuilder;
use crate::stupid_ai::prelude::IsHuman;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
    prelude::*,
};

pub struct BevyUiPlugin;

impl Plugin for BevyUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(WinitSettings::desktop_app())
            .add_plugins(DebugUiPlugin)
            .add_systems(Update, toggle_overlay)
            .add_systems(Startup, setup)
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
    for event in reader.read() {
        debug!("Received Event!");
        if let Ok(trade_card_list) = trade_card_list.get_single() {
            debug!("Trade Card List exists!");
            if let Ok(player_trade_cards) = player_trade_cards.get(event.player_entity) {
                debug!("Player Trade Cards: {:?}", player_trade_cards);
                let grouped_cards = player_trade_cards.trade_cards_grouped_by_value_and_type();
                let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
                let bg_color = Color::srgba(0.5, 0.5, 0.5, 0.25);

                let booga = UIBuilder::from_entity(commands, trade_card_list,true).build();
                // I doon't like it...
                commands = booga.1;

                for (value, type_map) in grouped_cards.iter() {
                    debug!("Value: {}", value);
                    // Create a container for each value
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
    let mut root_ui = UIBuilder::new(commands);
    root_ui = root_ui
        .add_component::<TradeCardUiRoot>()
        .block(Val::Percent(25.), Val::Percent(100.), bg_color)
        .add_text_child("Your trade cards!", font.clone(), 24.0, Some(Color::WHITE))
        .with_children(|b: &mut UIBuilder| {
            b.add_text_child("Gorf", font.clone(), 24.0, Some(Color::WHITE));
        });
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
