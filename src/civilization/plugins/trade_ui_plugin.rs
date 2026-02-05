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
use crate::civilization::PlayerCardStack;
use crate::stupid_ai::prelude::IsHuman;
use crate::{GameActivity, GameState};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::state::state::StateTransitionEvent;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct GameStateDisplay;

pub struct TradeUiPlugin;

impl Plugin for TradeUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(WinitSettings::desktop_app())
            .insert_resource(UiBuilderDefaults::new())
            .add_systems(OnEnter(GameActivity::StartGame), setup_trade_ui)
            .add_systems(Update, (
                handle_player_draws_cards,
                handle_trade_scroll_input,
                update_game_state_display,
            ));
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

pub fn setup_trade_ui(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_defaults: ResMut<UiBuilderDefaults>,
    player_trade_cards: Query<&PlayerTradeCards , With<IsHuman>>,
) {
    // // Spawn camera for UI rendering
    // commands.spawn((
    //     Camera2d,
    //     IsDefaultUiCamera,
    //     Projection::Orthographic(OrthographicProjection::default_2d()),
    //     Msaa::Off,
    // ));

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_defaults.base_font = font;
    ui_defaults.bg_color = BG_COLOR;
    ui_defaults.text_color = TEXT_COLOR;
    ui_defaults.font_size = 16.0;
    ui_defaults.border_color = BORDER_COLOR;
    ui_defaults.button_def = Some(ButtonPartial {
        border_radius: Some(BorderRadius::MAX),
        border_color: Some(BORDER_COLOR),
        ..default()
    });
    ui_defaults.node_def = Some(
        NodePartial::new()
            .border_all_px(2.0)
            .border_color(BORDER_COLOR)
            .border_radius_zero()
            .padding_all_px(4.0)
            .margin_all_px(2.0),
    );
    ui_defaults.text_justify = Some(Justify::Center);
    ui_defaults.text_line_break = Some(LineBreak::WordBoundary);

    let mut ui = UIBuilder::new(commands, Some(ui_defaults.clone()));

    ui.with_component::<TradeCardUiRoot>()
        .size_percent(50.0, 98.0)
        .display_flex()
        .flex_dir_row();
    

    // Left side: Trade cards grouped by pile value (scrollable)
    ui.with_child(|ui| {
        ui.size_percent(50.0, 100.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.3))
            .display_flex()
            .flex_dir_column()
            .with_overflow(Overflow::scroll_y())
            .insert(ScrollPosition::default())
            .with_component::<TradeCardList>();
        
        if let Ok(trade_cards) = player_trade_cards.single() {
            build_trade_card_list(ui, trade_cards);
        }
    });
    ui.with_child(|ui| {
        ui.size_percent(50.0, 100.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.3))
            .display_flex()
            .flex_dir_column();
        
        ui.add_text_child("Game State Info", None, Some(14.0), None);
        ui.with_child(|info| {
            info.width_percent(100.0)
                .height_px(60.0)
                .display_flex()
                .flex_dir_column()
                .padding_all_px(8.0)
                .with_component::<GameStateDisplay>();
            
            info.add_text_child("State: Playing", None, Some(12.0), None);
            info.add_text_child("Activity: StartGame", None, Some(12.0), None);
        });
    });

    let (_root, _commands) = ui.build();
}

pub fn build_trade_card(ui: &mut UIBuilder, stack: &PlayerCardStack) {
    let small_font_size = 9.0;
    let medium_font_size = 11.0;
    
    ui.with_child(|card| {
        card.width_percent(20.0)
            .height_percent(100.)
            .display_flex()
            .flex_dir_column()
            .justify_center()
            .align_items_center()
            .padding_all_px(2.0)
            .margin_all_px(2.0)
            .bg_color(Color::srgba(0.2, 0.2, 0.3, 0.8))
            .border_radius_all_px(4.0);
        
        if stack.is_commodity {
            card.add_text_child(stack.card_type.to_string(), None, Some(medium_font_size), None);
            card.add_text_child(format!("x{} = {}", stack.count, stack.suite_value), None, Some(small_font_size), None);
        } else {
            card.add_text_child(stack.card_type.to_string(), None, Some(medium_font_size), None);
            card.add_text_child(
                if stack.is_tradeable { "Tradeable" } else { "Non-Tradeable" },
                None,
                Some(small_font_size),
                None,
            );
        }
    });
}

pub fn build_trade_card_list(ui: &mut UIBuilder, trade_cards: &PlayerTradeCards) {
    let stacks = trade_cards.as_card_stacks_sorted_by_value();
    let row_count = 9f32;
    
    // Group stacks by pile value (1-9)
    for pile_value in 1..=9 {
        let pile_stacks: Vec<_> = stacks
            .iter()
            .filter(|s| s.card_type.value() == pile_value)
            .collect();
        
        if !pile_stacks.is_empty() {
            // Sort: commodities first, then calamities
            let mut sorted_stacks = pile_stacks.clone();
            sorted_stacks.sort_by_key(|s| if s.is_commodity { 0 } else { 1 });
            
            // Create a row for this pile
            ui.add_row(|row| {
                row.width_percent(100.0)
                    .height_percent(90.0 / row_count)
                    .justify_start()
                    .align_items_center()
                    .with_flex_shrink(0.0);
                
                // Pile label
                row.add_text_child(format!("{}:", pile_value), None, Some(12.0), None);
                
                // Cards in this pile
                for stack in sorted_stacks {
                    build_trade_card(row, stack);
                }
            });
        }
    }
}

/// Handle mouse wheel scroll input for scrollable containers
pub fn handle_trade_scroll_input(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scroll_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel in mouse_wheel_events.read() {
        let dy = match mouse_wheel.unit {
            MouseScrollUnit::Line => mouse_wheel.y * 20.0,
            MouseScrollUnit::Pixel => mouse_wheel.y,
        };

        // Apply scroll to hovered scrollable elements
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys() {
                if let Ok(mut scroll_position) = scroll_query.get_mut(*entity) {
                    scroll_position.y -= dy;
                    scroll_position.y = scroll_position.y.max(0.0);
                }
            }
        }
    }
}

fn update_game_state_display(
    commands: Commands,
    mut game_state_events: MessageReader<StateTransitionEvent<GameState>>,
    mut game_activity_events: MessageReader<StateTransitionEvent<GameActivity>>,
    display_query: Query<Entity, With<GameStateDisplay>>,
    ui_defaults: Res<UiBuilderDefaults>,
    current_state: Res<State<GameState>>,
    current_activity: Option<Res<State<GameActivity>>>,
) {
    let state_changed = game_state_events.read().count() > 0;
    let activity_changed = game_activity_events.read().count() > 0;
    
    if !state_changed && !activity_changed {
        return;
    }
    
    let Ok(display_entity) = display_query.single() else {
        return;
    };
    
    let state_text = format!("State: {:?}", current_state.get());
    let activity_text = match &current_activity {
        Some(activity) => format!("Activity: {:?}", activity.get()),
        None => "Activity: None".to_string(),
    };
    
    let mut ui = UIBuilder::start_from_entity(
        commands,
        display_entity,
        true,
        Some(ui_defaults.clone()),
    );
    
    ui.add_text_child(&state_text, None, Some(12.0), None);
    ui.add_text_child(&activity_text, None, Some(12.0), None);
    
    ui.build();
}