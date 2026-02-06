//! This example illustrates the various features of Bevy UI.

use crate::civilization::components::Faction;
use crate::civilization::concepts::acquire_trade_cards::trade_card_components::PlayerTradeCards;
use crate::civilization::concepts::acquire_trade_cards::trade_card_enums::{
    TradeCard, TradeCardTrait,
};
use crate::civilization::concepts::acquire_trade_cards::trade_card_events::HumanPlayerTradeCardsUpdated;
use crate::civilization::concepts::city_construction::city_construction_events::BuildCityCommand;
use crate::civilization::concepts::movement::movement_events::MoveTokenFromAreaToAreaCommand;
use crate::civilization::concepts::population_expansion::population_expansion_events::ExpandPopulationManuallyCommand;
use crate::civilization::concepts::trade::trade_components::{
    Collapsible, CollapseToggleButton, CollapsibleContent, TradeCardList, TradeCardUiRoot,
};
// Note: Collapsible, CollapseToggleButton, CollapsibleContent are used by the systems below
use crate::player::Player;
use bevy::platform::collections::HashMap;
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

#[derive(Component, Default)]
pub struct PlayerActivityListContainer;

#[derive(Component)]
pub struct PlayerActivityRow {
    pub player: Entity,
}

#[derive(Resource, Default)]
pub struct PlayerActivityLog {
    pub activities: HashMap<Entity, String>,
}

impl PlayerActivityLog {
    pub fn log(&mut self, player: Entity, activity: String) {
        self.activities.insert(player, activity);
    }
    
    pub fn get(&self, player: Entity) -> &str {
        self.activities.get(&player).map(|s| s.as_str()).unwrap_or("Waiting...")
    }
}

pub struct TradeUiPlugin;

impl Plugin for TradeUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(WinitSettings::desktop_app())
            .insert_resource(UiBuilderDefaults::new())
            .init_resource::<PlayerActivityLog>()
            .add_systems(OnEnter(GameActivity::StartGame), setup_trade_ui)
            .add_systems(Update, (
                handle_player_draws_cards,
                handle_trade_scroll_input,
                update_game_state_display,
                track_player_activities,
                update_player_activity_display,
                handle_collapse_toggle_button,
                update_collapsible_visibility,
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
    _player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
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
        .display_flex()
        .flex_dir_row()
        .padding_all_px(4.0);

    // Left side: Collapsible Trade Cards section
    ui.add_collapsible("Trade Cards", |cards_section| {
        cards_section
            .with_component::<TradeCardList>()
            .width_px(300.0)
            .height_px(400.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.7))
            .with_overflow(Overflow::scroll_y())
            .insert(ScrollPosition::default())
            .padding_all_px(4.0);
        
        if let Ok(trade_cards) = _player_trade_cards.single() {
            build_trade_card_list(cards_section, trade_cards);
        }
    });

    // Right side: Collapsible Game Info section
    ui.add_collapsible("Game Info", |info_section| {
        info_section
            .width_px(250.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.7))
            .padding_all_px(4.0);
        
        // Game State section
        info_section.add_text_child("Game State", None, Some(14.0), None);
        info_section.with_child(|state| {
            state
                .with_component::<GameStateDisplay>()
                .width_percent(100.0)
                .display_flex()
                .flex_dir_column()
                .padding_all_px(4.0)
                .margin(UiRect::bottom(Val::Px(8.0)));
            
            state.add_text_child("State: Playing", None, Some(10.0), None);
            state.add_text_child("Activity: StartGame", None, Some(10.0), None);
        });
        
        // Player Activity section
        info_section.add_text_child("Player Activity", None, Some(14.0), None);
        info_section.with_child(|list| {
            list.with_component::<PlayerActivityListContainer>()
                .width_percent(100.0)
                .height_px(300.0)
                .display_flex()
                .flex_dir_column()
                .padding_all_px(4.0)
                .with_overflow(Overflow::scroll_y())
                .insert(ScrollPosition::default());
        });
    });

    let (_root, _commands) = ui.build();
}

pub fn build_trade_card(ui: &mut UIBuilder, stack: &PlayerCardStack) {
    let small_font_size = 9.0;
    let medium_font_size = 11.0;
    
    ui.with_child(|card| {
        card.width_px(55.0)
            .height_px(38.0)
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
                    .height_px(44.0)
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
    
    ui.add_text_child(&state_text, None, Some(10.0), None);
    ui.add_text_child(&activity_text, None, Some(10.0), None);
    
    ui.build();
}

fn track_player_activities(
    mut activity_log: ResMut<PlayerActivityLog>,
    mut pop_exp_events: MessageReader<ExpandPopulationManuallyCommand>,
    mut movement_events: MessageReader<MoveTokenFromAreaToAreaCommand>,
    mut city_build_events: MessageReader<BuildCityCommand>,
    _player_names: Query<&Name>,
    area_names: Query<&Name>,
) {
    for event in pop_exp_events.read() {
        let area_name = area_names.get(event.area).map(|n| n.as_str()).unwrap_or("?");
        activity_log.log(
            event.player,
            format!("🌱 Expanding {} tokens to {}", event.number_of_tokens, area_name),
        );
    }
    
    for event in movement_events.read() {
        let from = area_names.get(event.source_area).map(|n| n.as_str()).unwrap_or("?");
        let to = area_names.get(event.target_area).map(|n| n.as_str()).unwrap_or("?");
        activity_log.log(
            event.player,
            format!("🚶 Moving {} from {} to {}", event.number_of_tokens, from, to),
        );
    }
    
    for event in city_build_events.read() {
        let area_name = area_names.get(event.area).map(|n| n.as_str()).unwrap_or("?");
        activity_log.log(
            event.player,
            format!("🏛️ Building city in {}", area_name),
        );
    }
}

fn update_player_activity_display(
    commands: Commands,
    activity_log: Res<PlayerActivityLog>,
    ui_defaults: Res<UiBuilderDefaults>,
    container_query: Query<Entity, With<PlayerActivityListContainer>>,
    players: Query<(Entity, &Name, &Faction), With<Player>>,
) {
    if !activity_log.is_changed() {
        return;
    }
    
    let Ok(container) = container_query.single() else {
        return;
    };
    
    let mut ui = UIBuilder::start_from_entity(
        commands,
        container,
        true,
        Some(ui_defaults.clone()),
    );
    
    for (player_entity, name, faction) in players.iter() {
        let activity = activity_log.get(player_entity);
        let faction_color = faction_to_color(faction);
        
        ui.with_child(|row| {
            row.width_percent(100.0)
                .height_px(36.0)
                .display_flex()
                .flex_dir_row()
                .align_items_center()
                .padding_all_px(4.0)
                .margin_all_px(2.0)
                .bg_color(Color::srgba(0.15, 0.15, 0.2, 0.8))
                .border_radius_all_px(4.0);
            
            row.with_child(|badge| {
                badge.width_px(12.0)
                    .height_px(12.0)
                    .bg_color(faction_color)
                    .border_radius_all_px(6.0)
                    .margin_all_px(6.0);
            });
            
            row.add_text_child(format!("{}: ", name), None, Some(13.0), Some(faction_color));
            row.add_text_child(activity, None, Some(12.0), None);
        });
    }
    
    ui.build();
}

fn faction_to_color(faction: &Faction) -> Color {
    use crate::civilization::enums::GameFaction;
    match faction.faction {
        GameFaction::Egypt => Color::srgb(0.9, 0.8, 0.3),
        GameFaction::Crete => Color::srgb(0.3, 0.6, 0.9),
        GameFaction::Africa => Color::srgb(0.6, 0.4, 0.2),
        GameFaction::Asia => Color::srgb(0.9, 0.5, 0.2),
        GameFaction::Assyria => Color::srgb(0.7, 0.2, 0.2),
        GameFaction::Babylon => Color::srgb(0.5, 0.3, 0.7),
        GameFaction::Illyria => Color::srgb(0.3, 0.7, 0.4),
        GameFaction::Iberia => Color::srgb(0.8, 0.6, 0.1),
        GameFaction::Thrace => Color::srgb(0.4, 0.5, 0.6),
    }
}

// ============================================================================
// COLLAPSIBLE UI SYSTEMS
// ============================================================================

/// Handle clicking the collapse/expand toggle button
pub fn handle_collapse_toggle_button(
    mut interaction_query: Query<
        (&Interaction, &CollapseToggleButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut collapsible_query: Query<&mut Collapsible>,
) {
    for (interaction, toggle_btn, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.6, 0.4));
                if let Ok(mut collapsible) = collapsible_query.get_mut(toggle_btn.target) {
                    collapsible.collapsed = !collapsible.collapsed;
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
            }
        }
    }
}

/// Update visibility of collapsible content based on collapsed state
pub fn update_collapsible_visibility(
    collapsible_query: Query<(Entity, &Collapsible), Changed<Collapsible>>,
    mut content_query: Query<(&CollapsibleContent, &mut Node)>,
    mut button_text_query: Query<(&CollapseToggleButton, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (collapsible_entity, collapsible) in collapsible_query.iter() {
        // Update content visibility
        for (content, mut node) in content_query.iter_mut() {
            if content.parent == collapsible_entity {
                node.display = if collapsible.collapsed {
                    Display::None
                } else {
                    Display::Flex
                };
            }
        }
        
        // Update button text
        for (toggle_btn, children) in button_text_query.iter_mut() {
            if toggle_btn.target == collapsible_entity {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = if collapsible.collapsed {
                            format!("▶ {}", collapsible.label)
                        } else {
                            format!("▼ {}", collapsible.label)
                        };
                    }
                }
            }
        }
    }
}

/// Helper function to spawn a collapsible section with content
/// Returns the content entity where children should be added
pub fn spawn_collapsible_section(
    commands: &mut Commands,
    label: &str,
    initially_collapsed: bool,
) -> (Entity, Entity) {
    let collapsible_entity = commands
        .spawn((
            if initially_collapsed {
                Collapsible::collapsed(label)
            } else {
                Collapsible::new(label)
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Auto,
                ..Default::default()
            },
        ))
        .id();
    
    // Spawn toggle button
    commands.entity(collapsible_entity).with_children(|parent| {
        parent
            .spawn((
                Button,
                CollapseToggleButton { target: collapsible_entity },
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..Default::default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            ))
            .with_child((
                Text::new(if initially_collapsed {
                    format!("▶ {}", label)
                } else {
                    format!("▼ {}", label)
                }),
                TextFont { font_size: 12.0, ..Default::default() },
                TextColor(Color::WHITE),
            ));
    });
    
    // Spawn content container
    let content_entity = commands
        .spawn((
            CollapsibleContent { parent: collapsible_entity },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: if initially_collapsed { Display::None } else { Display::Flex },
                ..Default::default()
            },
        ))
        .id();
    
    commands.entity(collapsible_entity).add_child(content_entity);
    
    (collapsible_entity, content_entity)
}