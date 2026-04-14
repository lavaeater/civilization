use crate::civilization::components::{Faction, PlayerAreas, PlayerCities, TokenStock, Treasury};
use crate::civilization::concepts::ships::ShipStock;
use crate::civilization::concepts::*;
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::{GameActivity, GameState};
use bevy::prelude::*;
use bevy::state::state::StateTransitionEvent;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component, Default)]
pub struct TradeCardUiRoot;

#[derive(Component, Default)]
pub struct TradeCardList;

/// Marker for the trade phase root UI (used elsewhere in the codebase).
#[derive(Component, Default)]
pub struct TradePhaseUiRoot;

/// Container rebuilt by `update_hud_dynamic` with human-player stats.
#[derive(Component, Default)]
pub struct PlayerInfoDisplay;

/// Container rebuilt by `update_hud_dynamic` with the census-order table.
#[derive(Component, Default)]
pub struct CensusDisplay;

/// Container rebuilt by `update_hud_dynamic` with current state/activity/round.
#[derive(Component, Default)]
pub struct ActivityDisplay;

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct TradeUiPlugin;

impl Plugin for TradeUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LavaTheme::default())
            .add_systems(OnEnter(GameActivity::StartGame), setup_trade_ui)
            .add_systems(
                Update,
                (handle_player_draws_cards, update_hud_dynamic),
            );
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_trade_ui(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_theme: ResMut<LavaTheme>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    ui_theme.text.font = font;
    ui_theme.text.label_size = 14.0;
    ui_theme.button.border_radius = BorderRadius::MAX;

    let mut ui = UIBuilder::new(commands, Some(ui_theme.clone()));

    // HUD strip anchored to the bottom of the screen
    ui.component::<TradeCardUiRoot>()
        .absolute_position()
        .bottom(Val::Px(0.0))
        .left(Val::Px(0.0))
        .display_flex()
        .flex_row()
        .align_items_end()
        .padding_all_px(4.0)
        .gap_px(4.0);

    // ── Card 1: Player Info ───────────────────────────────────────────────────
    ui.with_collapsible("Player Info", false, |section| {
        section
            .component::<PlayerInfoDisplay>()
            .width_px(220.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.85))
            .padding_all_px(6.0);
        // Content filled by update_hud_dynamic
    });

    // ── Card 2: Trade Cards ───────────────────────────────────────────────────
    ui.with_collapsible("Trade Cards", false, |section| {
        section
            .component::<TradeCardList>()
            .width_px(320.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.85))
            .padding_all_px(6.0);

        if let Ok(trade_cards) = player_trade_cards.single() {
            build_compact_trade_card_list(section, trade_cards);
        }
    });

    // ── Card 3: Game State (census order) ─────────────────────────────────────
    ui.with_collapsible("Game State", false, |section| {
        section
            .component::<CensusDisplay>()
            .width_px(380.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.85))
            .padding_all_px(6.0);
        // Content filled by update_hud_dynamic
    });

    // ── Card 4: Game Activity ─────────────────────────────────────────────────
    ui.with_collapsible("Game Activity", false, |section| {
        section
            .component::<ActivityDisplay>()
            .width_px(200.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.85))
            .padding_all_px(6.0);
        // Content filled by update_hud_dynamic
    });

    ui.build();
}

// ── Trade card rebuild ────────────────────────────────────────────────────────

fn handle_player_draws_cards(
    mut reader: MessageReader<HumanPlayerTradeCardsUpdated>,
    commands: Commands,
    ui_theme: Res<LavaTheme>,
    trade_card_list: Query<Entity, With<TradeCardList>>,
    player_trade_cards: Query<&PlayerTradeCards, With<IsHuman>>,
) {
    let mut new_commands = commands;
    for event in reader.read() {
        if let Ok(list_entity) = trade_card_list.single()
            && let Ok(trade_cards) = player_trade_cards.get(event.player_entity)
        {
            let mut builder = UIBuilder::start_from_entity(
                new_commands,
                list_entity,
                true,
                Some(ui_theme.clone()),
            );
            build_compact_trade_card_list(&mut builder, trade_cards);
            new_commands = builder.build().1;
        }
    }
}

/// Compact one-row-per-card-type list:  `Val  Name              ×N  = Total`
pub fn build_compact_trade_card_list(ui: &mut UIBuilder, trade_cards: &PlayerTradeCards) {
    let stacks = trade_cards.as_card_stacks_sorted_by_value();
    if stacks.is_empty() {
        ui.add_text_child("(no cards)", Some(TextStyle::size_color(13.0, Color::srgb(0.5, 0.5, 0.5))));
        return;
    }

    // Header row
    ui.add_row(|row| {
        row.gap_px(0.0);
        table_cell(row, "Val",   30.0, Color::srgb(0.6, 0.6, 0.6), 12.0);
        table_cell(row, "Name",  140.0, Color::srgb(0.6, 0.6, 0.6), 12.0);
        table_cell(row, "×N",    36.0, Color::srgb(0.6, 0.6, 0.6), 12.0);
        table_cell(row, "Total", 60.0, Color::srgb(0.6, 0.6, 0.6), 12.0);
    });

    for stack in &stacks {
        let value_color = if stack.is_commodity {
            Color::srgb(0.9, 0.9, 0.5)
        } else {
            Color::srgb(0.9, 0.4, 0.4)
        };
        let count_str = format!("×{}", stack.count);
        let total_str = if stack.is_commodity {
            format!("= {}", stack.suite_value)
        } else {
            "—".to_string()
        };

        ui.add_row(|row| {
            row.gap_px(0.0);
            table_cell(row, stack.card_type.value().to_string(), 30.0, value_color, 13.0);
            table_cell(row, stack.card_type.to_string(),         140.0, Color::WHITE, 13.0);
            table_cell(row, count_str,                            36.0, Color::srgb(0.7, 0.9, 0.7), 13.0);
            table_cell(row, total_str,                            60.0, Color::srgb(0.7, 0.8, 1.0), 13.0);
        });
    }
}

// ── HUD dynamic sections ──────────────────────────────────────────────────────

fn update_hud_dynamic(
    commands: Commands,
    mut game_state_events: MessageReader<StateTransitionEvent<GameState>>,
    mut game_activity_events: MessageReader<StateTransitionEvent<GameActivity>>,
    player_info_query: Query<Entity, With<PlayerInfoDisplay>>,
    census_query: Query<Entity, With<CensusDisplay>>,
    activity_query: Query<Entity, With<ActivityDisplay>>,
    ui_theme: Res<LavaTheme>,
    current_state: Res<State<GameState>>,
    current_activity: Option<Res<State<GameActivity>>>,
    game_info: Res<GameInfoAndStuff>,
    // Human player stats
    human_query: Query<
        (&TokenStock, &Treasury, &ShipStock, &PlayerAreas, &PlayerCities),
        With<IsHuman>,
    >,
    // All players for census
    player_query: Query<
        (&Name, &PlayerAreas, &PlayerCities, &PlayerTradeCards, &Faction, Has<IsHuman>),
        With<Player>,
    >,
) {
    let state_changed = game_state_events.read().count() > 0;
    let activity_changed = game_activity_events.read().count() > 0;
    if !state_changed && !activity_changed {
        return;
    }

    let mut cmds = commands;

    // ── Player Info ──────────────────────────────────────────────────────────
    if let Ok(info_entity) = player_info_query.single() {
        let mut ui = UIBuilder::start_from_entity(cmds, info_entity, true, Some(ui_theme.clone()));

        if let Ok((token_stock, treasury, ship_stock, player_areas, player_cities)) =
            human_query.single()
        {
            let pop = player_areas.total_population();
            let cities = player_cities.number_of_cities();
            let tokens = token_stock.tokens_in_stock();
            let treasury_count = treasury.tokens_in_treasury();
            let ships = ship_stock.count_in_stock();

            info_row(&mut ui, "Population:",   pop.to_string());
            info_row(&mut ui, "Cities:",        cities.to_string());
            info_row(&mut ui, "Tokens in stock:", tokens.to_string());
            info_row(&mut ui, "Treasury:",      treasury_count.to_string());
            info_row(&mut ui, "Ships in stock:", format!("{} / {}", ships, ShipStock::MAX_SHIPS));
        } else {
            ui.add_text_child("No human player", None);
        }

        cmds = ui.build().1;
    }

    // ── Census Display ───────────────────────────────────────────────────────
    if let Ok(census_entity) = census_query.single() {
        let mut ui = UIBuilder::start_from_entity(cmds, census_entity, true, Some(ui_theme.clone()));

        // Header row
        ui.add_row(|row| {
            row.gap_px(0.0);
            table_cell(row, "#",       22.0,  Color::srgb(0.6, 0.6, 0.6), 11.0);
            table_cell(row, "Player",  160.0, Color::srgb(0.6, 0.6, 0.6), 11.0);
            table_cell(row, "Pop",      44.0, Color::srgb(0.6, 0.6, 0.6), 11.0);
            table_cell(row, "Cit",      44.0, Color::srgb(0.6, 0.6, 0.6), 11.0);
            table_cell(row, "Crd",      44.0, Color::srgb(0.6, 0.6, 0.6), 11.0);
        });

        for (i, player_entity) in game_info.census_order.iter().enumerate() {
            if let Ok((name, player_areas, player_cities, player_trade_cards, faction, is_human)) =
                player_query.get(*player_entity)
            {
                let pop = player_areas.total_population();
                let cities = player_cities.number_of_cities();
                let cards = player_trade_cards.number_of_trade_cards();
                let color = faction_to_color(faction);
                let display_name = if is_human {
                    format!("{} (YOU)", name)
                } else {
                    name.to_string()
                };

                ui.add_row(|row| {
                    row.gap_px(0.0);
                    table_cell(row, (i + 1).to_string(), 22.0,  color, 12.0);
                    table_cell(row, display_name,         160.0, color, 12.0);
                    table_cell(row, pop.to_string(),       44.0, Color::WHITE, 12.0);
                    table_cell(row, cities.to_string(),    44.0, Color::WHITE, 12.0);
                    table_cell(row, cards.to_string(),     44.0, Color::WHITE, 12.0);
                });
            }
        }

        cmds = ui.build().1;
    }

    // ── Activity Display ─────────────────────────────────────────────────────
    if let Ok(activity_entity) = activity_query.single() {
        let mut ui =
            UIBuilder::start_from_entity(cmds, activity_entity, true, Some(ui_theme.clone()));

        ui.add_text_child(
            format!("Round: {}", game_info.round),
            Some(TextStyle::size_color(14.0, Color::srgb(0.9, 0.9, 0.5))),
        );
        ui.add_text_child(
            format!("State: {:?}", current_state.get()),
            Some(TextStyle::size(13.0)),
        );
        if let Some(activity) = &current_activity {
            ui.add_text_child(
                format!("Activity: {:?}", activity.get()),
                Some(TextStyle::size_color(13.0, Color::srgb(0.5, 0.9, 0.9))),
            );
        }

        ui.build();
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Spawn a fixed-width text cell inside a table row.
fn table_cell(row: &mut UIBuilder, text: impl Into<String>, width: f32, color: Color, size: f32) {
    row.with_child(|c| {
        c.width_px(width)
            .with_text(text, Some(TextStyle::size_color(size, color)));
    });
}

/// Spawn a two-column key/value info row.
fn info_row(ui: &mut UIBuilder, label: &str, value: String) {
    ui.add_row(|row| {
        row.gap_px(4.0);
        row.with_child(|c| {
            c.width_px(130.0)
                .with_text(label, Some(TextStyle::size_color(13.0, Color::srgb(0.6, 0.6, 0.6))));
        });
        row.with_child(|c| {
            c.with_text(value, Some(TextStyle::size(13.0)));
        });
    });
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

// ── Old helpers kept for civ_cards UI compatibility ───────────────────────────

pub fn build_trade_card(ui: &mut UIBuilder, stack: &PlayerCardStack) {
    let small_font_size = 12.0;
    let medium_font_size = 18.0;

    ui.with_child(|card| {
        card.width_px(120.0)
            .height_px(80.0)
            .display_flex()
            .flex_column()
            .justify_center()
            .align_items_center()
            .padding_all_px(2.0)
            .margin_all_px(2.0)
            .bg_color(Color::srgba(0.2, 0.2, 0.3, 0.8))
            .border_radius_all_px(4.0);

        if stack.is_commodity {
            card.add_text_child(
                stack.card_type.to_string(),
                Some(TextStyle::size(medium_font_size)),
            );
            card.add_text_child(
                format!("x{} = {}", stack.count, stack.suite_value),
                Some(TextStyle::size(small_font_size)),
            );
        } else {
            card.add_text_child(
                stack.card_type.to_string(),
                Some(TextStyle::size(medium_font_size)),
            );
            card.add_text_child(
                if stack.is_tradeable { "Tradeable" } else { "Non-Tradeable" },
                Some(TextStyle::size(small_font_size)),
            );
        }
    });
}

pub fn build_trade_card_list(ui: &mut UIBuilder, trade_cards: &PlayerTradeCards) {
    let stacks = trade_cards.as_card_stacks_sorted_by_value();

    for pile_value in 1..=9 {
        let pile_stacks: Vec<_> = stacks
            .iter()
            .filter(|s| s.card_type.value() == pile_value)
            .collect();

        if !pile_stacks.is_empty() {
            let mut sorted_stacks = pile_stacks.clone();
            sorted_stacks.sort_by_key(|s| if s.is_commodity { 0 } else { 1 });

            ui.add_row(|row| {
                row.width_percent(100.0)
                    .height_px(44.0)
                    .justify_start()
                    .align_items_center()
                    .with_flex_shrink(0.0);

                row.add_text_child(format!("{}:", pile_value), Some(TextStyle::size(24.0)));

                for stack in sorted_stacks {
                    build_trade_card(row, stack);
                }
            });
        }
    }
}
