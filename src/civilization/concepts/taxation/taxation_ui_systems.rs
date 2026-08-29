use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};

use crate::civilization::Z_DIALOG;
use crate::civilization::components::PlayerCities;
use crate::civilization::concepts::taxation::taxation_components::{
    AwaitingCoinageRateSelection, NeedsToPayTaxes,
};
use crate::stupid_ai::IsHuman;

/// Root node of the Coinage tax-rate picker, shown to a human player who
/// holds Coinage while `AwaitingCoinageRateSelection` is present (rule 19.2).
#[derive(Component, Default)]
pub struct CoinageRateUiRoot;

/// Spawn the tax-rate picker when a human gets `AwaitingCoinageRateSelection`.
pub fn spawn_coinage_rate_ui(
    human_waiting: Query<Entity, (With<IsHuman>, Added<AwaitingCoinageRateSelection>)>,
    existing_ui: Query<Entity, With<CoinageRateUiRoot>>,
    commands: Commands,
    theme: Res<LavaTheme>,
) {
    if !existing_ui.is_empty() || human_waiting.iter().next().is_none() {
        return;
    }

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<CoinageRateUiRoot>()
        .absolute_position()
        .top(Val::Px(20.0))
        .left(Val::Percent(50.0))
        .display_flex()
        .flex_column()
        .align_items_center()
        .padding_all_px(10.0)
        .gap_px(8.0)
        .z_index(Z_DIALOG)
        .bg_color(Color::srgba(0.05, 0.1, 0.15, 0.93));

    ui.add_text_child(
        "Coinage: Set This Round's Tax Rate",
        Some(TextStyle::size_color(20.0, Color::srgb(0.9, 0.8, 0.3))),
    );
    ui.add_text_child(
        "Choose how many tokens per city your cities pay in taxes this round",
        Some(TextStyle::size_color(13.0, Color::srgb(0.7, 0.7, 0.7))),
    );

    ui.with_child(|row| {
        row.display_flex().flex_row().gap_px(12.0);

        for rate in [1usize, 2, 3] {
            row.add_button_observe(
                format!("{rate} / city"),
                |btn| {
                    btn.size_px(120.0, 44.0);
                },
                move |_: On<Activate>,
                      mut commands: Commands,
                      human_waiting: Query<
                    (Entity, &PlayerCities),
                    (With<IsHuman>, With<AwaitingCoinageRateSelection>),
                >| {
                    let Ok((player, cities)) = human_waiting.single() else {
                        return;
                    };
                    let tokens_owed = cities.number_of_cities() * rate;
                    info!(
                        "[TAXATION] Human chose Coinage rate {rate}/city ({tokens_owed} tokens owed)"
                    );
                    commands
                        .entity(player)
                        .remove::<AwaitingCoinageRateSelection>()
                        .insert(NeedsToPayTaxes::new(tokens_owed));
                },
            );
        }
    });

    ui.build();
}

/// Despawn the picker once the human's choice has been applied.
pub fn cleanup_coinage_rate_ui(
    mut commands: Commands,
    ui_root: Query<Entity, With<CoinageRateUiRoot>>,
    human_waiting: Query<Entity, (With<IsHuman>, With<AwaitingCoinageRateSelection>)>,
) {
    if !ui_root.is_empty() && human_waiting.is_empty() {
        for entity in &ui_root {
            commands.entity(entity).despawn();
        }
    }
}

/// Safety net: force-despawn the picker on `CollectTaxes`'s exit, in case the
/// phase ends while it's still open.
pub fn cleanup_coinage_rate_ui_on_exit(
    mut commands: Commands,
    ui_root: Query<Entity, With<CoinageRateUiRoot>>,
) {
    for entity in &ui_root {
        commands.entity(entity).despawn();
    }
}
