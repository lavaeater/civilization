use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{InteractionPalette, LavaTheme, TextStyle, UIBuilder};

use crate::civilization::Z_DIALOG;
use crate::civilization::concepts::acquire_trade_cards::TradeCardTrait;
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_systems::COMMODITY_CARD_HAND_LIMIT;
use crate::civilization::concepts::shed_commodity_cards::shed_commodity_cards_ui_components::*;
use crate::stupid_ai::IsHuman;

use super::shed_commodity_cards_components::AwaitingCommodityShedSelection;

/// Spawn the shed panel when the human player gets `AwaitingCommodityShedSelection`.
pub fn spawn_commodity_shed_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingCommodityShedSelection>)>,
    existing_ui: Query<Entity, With<CommodityShedUiRoot>>,
    shed_state: Res<CommodityShedSelectionState>,
    commands: Commands,
    theme: Res<LavaTheme>,
) {
    if !existing_ui.is_empty() || shed_state.player.is_none() {
        return;
    }
    if human_waiting.iter().next().is_none() {
        return;
    }

    let must_discard = shed_state.must_discard;
    let holdings = shed_state.holdings.clone();

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<CommodityShedUiRoot>()
        .absolute_position()
        .bottom(Val::Px(20.0))
        .left(Val::Percent(50.0))
        .display_flex()
        .flex_column()
        .align_items_center()
        .padding_all_px(10.0)
        .gap_px(6.0)
        .z_index(Z_DIALOG)
        .bg_color(Color::srgba(0.05, 0.1, 0.15, 0.93));

    ui.add_text_child(
        "Shed Commodity Cards",
        Some(TextStyle::size_color(22.0, Color::srgb(0.4, 0.8, 1.0))),
    );
    ui.add_text_child(
        format!(
            "Hand exceeds the {COMMODITY_CARD_HAND_LIMIT}-card limit -- choose {must_discard} card{} to discard",
            if must_discard == 1 { "" } else { "s" }
        ),
        Some(TextStyle::size_color(14.0, Color::srgb(0.7, 0.7, 0.7))),
    );

    ui.foreach_child(holdings, |row, (card, held)| {
        row.display_flex().flex_row().align_items_center().gap_px(8.0);

        row.with_child(|c| {
            c.with_text(
                format!("{card} (value {}) x{held}", card.value()),
                Some(TextStyle::size_color(14.0, Color::srgb(0.9, 0.9, 0.9))),
            )
            .width_px(220.0);
        });

        row.add_button_observe(
            "-",
            |btn| {
                btn.size_px(28.0, 28.0);
            },
            move |_: On<Activate>, mut s: ResMut<CommodityShedSelectionState>| {
                s.decrement(card);
            },
        );

        row.with_child(|c| {
            c.insert(CommodityShedCountText(card))
                .with_text(
                    "0",
                    Some(TextStyle::size_color(16.0, Color::srgb(1.0, 1.0, 0.7))),
                )
                .width_px(24.0);
        });

        row.add_button_observe(
            "+",
            |btn| {
                btn.size_px(28.0, 28.0);
            },
            move |_: On<Activate>, mut s: ResMut<CommodityShedSelectionState>| {
                s.increment(card);
            },
        );
    });

    ui.with_child(|c| {
        c.component::<CommodityShedProgressText>().with_text(
            format!("0 / {must_discard} selected"),
            Some(TextStyle::size_color(16.0, Color::srgb(0.8, 0.8, 0.8))),
        );
    });

    let confirm_color = Color::srgb(0.35, 0.35, 0.35);
    ui.add_button_observe(
        "Confirm",
        |btn| {
            btn.size_px(160.0, 40.0)
                .insert(CommodityShedConfirmButton)
                .bg_color(confirm_color);
        },
        |_: On<Activate>,
         mut commands: Commands,
         human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingCommodityShedSelection>)>,
         shed_state: Res<CommodityShedSelectionState>| {
            if !shed_state.is_complete() {
                return;
            }
            if let Ok(player) = human_waiting.single() {
                info!(
                    "[SHED UI] Human confirmed: {} card(s) selected for discard",
                    shed_state.total_selected()
                );
                commands
                    .entity(player)
                    .remove::<AwaitingCommodityShedSelection>();
            }
        },
    );

    ui.build();
}

/// Keep the shed panel's per-card counts, progress text, and Confirm button in
/// sync with `CommodityShedSelectionState`.
pub fn update_commodity_shed_ui(
    shed_state: Res<CommodityShedSelectionState>,
    mut count_texts: Query<(&CommodityShedCountText, &mut Text)>,
    mut progress_text: Query<
        &mut Text,
        (With<CommodityShedProgressText>, Without<CommodityShedCountText>),
    >,
    mut confirm_button: Query<
        (&mut BackgroundColor, &mut InteractionPalette),
        With<CommodityShedConfirmButton>,
    >,
) {
    if !shed_state.is_changed() {
        return;
    }

    for (tag, mut text) in &mut count_texts {
        **text = shed_state.selected_count(tag.0).to_string();
    }

    if let Ok(mut t) = progress_text.single_mut() {
        **t = format!(
            "{} / {} selected",
            shed_state.total_selected(),
            shed_state.must_discard
        );
    }

    let color = if shed_state.is_complete() {
        Color::srgb(0.2, 0.5, 0.2)
    } else {
        Color::srgb(0.35, 0.35, 0.35)
    };
    if let Ok((mut bg, mut palette)) = confirm_button.single_mut() {
        *bg = BackgroundColor(color);
        palette.none = color;
    }
}

/// Despawn the shed panel when no human is waiting and the state has been
/// cleared (i.e. `apply_commodity_shed` has applied the confirmed result).
pub fn cleanup_commodity_shed_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<CommodityShedUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingCommodityShedSelection>)>,
    shed_state: Res<CommodityShedSelectionState>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() && shed_state.player.is_none() {
        for entity in ui_root.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Safety net: force-despawn the panel and clear selection state on phase
/// exit, in case the phase ends (e.g. game over) while the panel is open.
pub fn cleanup_commodity_shed_ui_on_exit(
    mut commands: Commands,
    ui_root: Query<Entity, With<CommodityShedUiRoot>>,
    mut shed_state: ResMut<CommodityShedSelectionState>,
) {
    for entity in ui_root.iter() {
        commands.entity(entity).despawn();
    }
    shed_state.clear();
}
