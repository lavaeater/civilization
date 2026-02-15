use crate::civilization::components::{Faction, PlayerAreas, PlayerCities};
use crate::civilization::concepts::*;
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::{GameActivity, GameState};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::state::StateTransitionEvent;
use lava_ui_builder::{UIBuilder, LavaTheme};

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
            .insert_resource(LavaTheme::default())
            .init_resource::<PlayerActivityLog>()
            .add_systems(OnEnter(GameActivity::StartGame), setup_trade_ui)
            .add_systems(Update, (
                handle_player_draws_cards,
                update_game_state_display,
                track_player_activities,
                update_player_activity_display,
            ));
    }
}

fn _commodity_card_max_set_size(card: TradeCard) -> usize {
    match card.value() {
        1..=4 => 8,
        5 => 7,
        6 => 6,
        7 => 5,
        8 | 9 => 4,
        _ => 8,
    }
}

fn _add_commodity_card(
    b: &mut UIBuilder,
    card_type: TradeCard,
    count: usize,
    theme: &LavaTheme,
) {
    let active_index = count.clamp(1, _commodity_card_max_set_size(card_type));
    let max_set_size = _commodity_card_max_set_size(card_type);
    let highlight_bg = Color::srgba(1.0, 1.0, 1.0, 0.25);
    let normal_bg = Color::srgba(0.0, 0.0, 0.0, 0.0);
    let highlight_border = Color::srgba(1.0, 1.0, 1.0, 0.85);
    let card_color = Color::srgba(0.2, 0.2, 0.3, 0.8);
    let border_color = theme.border_color;
    let text_color = theme.text.label_color;
    let font = theme.text.font.clone();

    b.size_px(160., 240.).bg_color(card_color)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .border_all_px(2.0, border_color)
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
                .foreach_child(1..=max_set_size.min(4), |b: &mut UIBuilder, n| {
                    let is_active = n == active_index;
                    let bg = if is_active { highlight_bg } else { normal_bg };
                    let bc = if is_active {
                        highlight_border
                    } else {
                        border_color
                    };
                    b.size_px(18., 18.).bg_color(bg)
                        .border_all_px(1., bc)
                        // .align_content(AlignContent::Center)
                        // .align_items(AlignItems::Center)
                        // .justify_content(JustifyContent::Center)
                        // .justify_items(JustifyItems::Center)
                        .with_text(
                            format!("{}", (n * n) * card_type.value()),
                            Some(font.clone()),
                            Some(18.0),
                            Some(text_color),
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

fn _setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_theme: ResMut<LavaTheme>,
    human_players: Query<Entity, With<IsHuman>>,
    mut pulled_card_event_writer: MessageWriter<HumanPlayerTradeCardsUpdated>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_theme.text.font = font.clone();
    ui_theme.text.label_size = 18.0;
    ui_theme.button.border_radius = BorderRadius::MAX;

    let mut root_ui = UIBuilder::new(commands, Some(ui_theme.clone()));

    root_ui
        .component::<TradeCardUiRoot>()
        .display_flex().flex_column().size(Val::Percent(25.), Val::Percent(100.))
        .text_node("Your trade cards!")
        .child()
        .display_flex().flex_column().size(Val::Percent(100.), Val::Percent(100.))
        .component::<TradeCardList>();

    let (_root_entity, _commands) = root_ui.build();
    for human_player in human_players.iter() {
        pulled_card_event_writer.write(HumanPlayerTradeCardsUpdated::new(human_player));
    }
}

fn handle_player_draws_cards(
    mut reader: MessageReader<HumanPlayerTradeCardsUpdated>,
    commands: Commands,
    ui_theme: Res<LavaTheme>,
    trade_card_list: Query<Entity, With<TradeCardList>>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let mut new_commands = commands;
    for event in reader.read() {
        if let Ok(trade_card_list_entity) = trade_card_list.single() {
            if let Ok(player_trade_cards) = player_trade_cards.get(event.player_entity) {
                let mut ui_builder = UIBuilder::start_from_entity(
                    new_commands,
                    trade_card_list_entity,
                    true,
                    Some(ui_theme.clone()),
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
    mut ui_theme: ResMut<LavaTheme>,
    _player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_theme.text.font = font;
    ui_theme.text.label_size = 24.0;
    ui_theme.button.border_radius = BorderRadius::MAX;

    let mut ui = UIBuilder::new(commands, Some(ui_theme.clone()));

    ui.component::<TradeCardUiRoot>()
        .display_flex()
        .flex_dir_row()
        .padding_all_px(4.0);

    // Left side: Collapsible Trade Cards section
    ui.add_collapsible("Trade Cards", |cards_section| {
        cards_section
            .component::<TradeCardList>()
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
            .width_px(450.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.7))
            .padding_all_px(4.0);
        
        // Game State section
        info_section.add_text_child("Game State", None, None, None);
        info_section.with_child(|state| {
            state
                .component::<GameStateDisplay>()
                .width_percent(100.0)
                .display_flex()
                .flex_dir_column()
                .padding_all_px(4.0)
                .margin(UiRect::bottom(Val::Px(8.0)));
            
            state.add_text_child("State: Playing", None, None, None);
            state.add_text_child("Activity: StartGame", None, None, None);
        });
        
        // Player Activity section
        info_section.add_text_child("Player Activity", None, None, None);
        info_section.with_child(|list| {
            list.component::<PlayerActivityListContainer>()
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
    let small_font_size = 12.0;
    let medium_font_size = 18.0;
    
    ui.with_child(|card| {
        card.width_px(120.0)
            .height_px(80.0)
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
                row.add_text_child(format!("{}:", pile_value), None, Some(24.0), None);
                
                // Cards in this pile
                for stack in sorted_stacks {
                    build_trade_card(row, stack);
                }
            });
        }
    }
}

fn update_game_state_display(
    commands: Commands,
    mut game_state_events: MessageReader<StateTransitionEvent<GameState>>,
    mut game_activity_events: MessageReader<StateTransitionEvent<GameActivity>>,
    display_query: Query<Entity, With<GameStateDisplay>>,
    ui_theme: Res<LavaTheme>,
    current_state: Res<State<GameState>>,
    current_activity: Option<Res<State<GameActivity>>>,
    game_info: Res<GameInfoAndStuff>,
    player_query: Query<(&Name, &PlayerAreas, &PlayerCities, &PlayerTradeCards, &Faction, Has<IsHuman>), With<Player>>,
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
    let round_text = format!("Round: {}", game_info.round);
    
    let mut ui = UIBuilder::start_from_entity(
        commands,
        display_entity,
        true,
        Some(ui_theme.clone()),
    );
    
    ui.add_text_child(&state_text, None, Some(18.0), None);
    ui.add_text_child(&activity_text, None, Some(18.0), None);
    ui.add_text_child(&round_text, None, Some(18.0), None);

    // Census order display
    ui.add_text_child("Census Order:", None, Some(16.0), Some(Color::srgb(1.0, 0.8, 0.0)));
    for (i, player_entity) in game_info.census_order.iter().enumerate() {
        if let Ok((name, player_areas, player_cities, player_trade_cards, faction, is_human)) = player_query.get(*player_entity) {
            let pop = player_areas.total_population();
            let cities = player_cities.number_of_cities();
            let cards = player_trade_cards.number_of_trade_cards();
            let faction_color = faction_to_color(faction);
            let human_marker = if is_human { " (YOU)" } else { "" };
            let census_line = format!("{}. {}{} - Pop: {} | Cities: {} | Cards: {}", i + 1, name, human_marker, pop, cities, cards);
            ui.add_text_child(&census_line, None, Some(14.0), Some(faction_color));
        }
    }
    
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
    ui_theme: Res<LavaTheme>,
    container_query: Query<Entity, With<PlayerActivityListContainer>>,
    players: Query<(Entity, &Name, &Faction, Has<IsHuman>), With<Player>>,
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
        Some(ui_theme.clone()),
    );
    
    for (player_entity, name, faction, is_human) in players.iter() {
        let activity = activity_log.get(player_entity);
        let faction_color = faction_to_color(faction);
        let display_name = if is_human {
            format!("👤 {} (YOU)", name)
        } else {
            name.to_string()
        };
        
        ui.with_child(|row| {
            row.width_percent(100.0)
                .height_px(80.0)
                .display_flex()
                .flex_dir_row()
                .align_items_center()
                .padding_all_px(4.0)
                .margin_all_px(2.0)
                .bg_color(Color::srgba(0.15, 0.15, 0.2, 0.8))
                .border_radius_all_px(4.0);
            
            row.with_child(|badge| {
                badge.width_px(24.0)
                    .height_px(24.0)
                    .bg_color(faction_color)
                    .border_radius_all_px(6.0)
                    .margin_all_px(6.0);
            });
            
            row.add_text_child(format!("{}: ", display_name), None, Some(18.0), Some(faction_color));
            row.add_text_child(activity, None, Some(18.0), None);
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

#[derive(Component, Default)]
pub struct TradeCardUiRoot;

#[derive(Component, Default)]
pub struct TradeCardList;

/// Marker for the trade phase root UI
#[derive(Component, Default)]
pub struct TradePhaseUiRoot;