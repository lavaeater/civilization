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

/// Tags a rate-picker button with the rate it sets, so tests (and any future
/// code) can find a specific button without depending on its label text.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoinageRateButton(pub usize);

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
                move |btn| {
                    btn.size_px(120.0, 44.0).insert(CoinageRateButton(rate));
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

// ─── Unit tests ────────────────────────────────────────────────────────────────
//
// These trigger the real `Activate` observer registered on the spawned button
// entity -- the same event `bevy_ui_widgets` fires on an actual click -- rather
// than calling the handler's closure logic by hand. That's the difference
// between testing "what the button is supposed to do" and testing "what
// clicking the button actually does": a wiring mistake (wrong entity, button
// never spawned, marker never removed) would pass the former and fail the
// latter. Since the handlers mutate the world via `Commands`, `world.flush()`
// is required after `world.trigger()` to apply the deferred changes -- the
// same as an `App::update()` tick would do for a real click.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::components::PlayerCities;
    use bevy::ecs::system::RunSystemOnce;

    fn spawn_human_awaiting_with_cities(world: &mut World, city_count: usize) -> Entity {
        let mut cities = PlayerCities::default();
        for _ in 0..city_count {
            let area = world.spawn_empty().id();
            let city = world.spawn_empty().id();
            cities.build_city_in_area(area, city);
        }
        world
            .spawn((IsHuman, cities, AwaitingCoinageRateSelection))
            .id()
    }

    fn find_rate_button(world: &mut World, rate: usize) -> Entity {
        world
            .query::<(Entity, &CoinageRateButton)>()
            .iter(world)
            .find(|(_, b)| b.0 == rate)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("no button found for rate {rate}"))
    }

    #[test]
    fn clicking_a_rate_button_computes_tax_and_clears_awaiting() {
        let mut world = World::new();
        world.init_resource::<LavaTheme>();
        let player = spawn_human_awaiting_with_cities(&mut world, 3);

        world.run_system_once(spawn_coinage_rate_ui).unwrap();
        assert!(
            !world
                .query::<&CoinageRateUiRoot>()
                .iter(&world)
                .collect::<Vec<_>>()
                .is_empty(),
            "picker should have spawned"
        );

        let button = find_rate_button(&mut world, 3);
        world.trigger(Activate { entity: button });
        world.flush();

        let needs = world
            .get::<NeedsToPayTaxes>(player)
            .expect("clicking the button should insert NeedsToPayTaxes");
        assert_eq!(needs.tokens_owed, 9, "3 cities at rate 3 = 9 tokens owed");
        assert!(
            world.get::<AwaitingCoinageRateSelection>(player).is_none(),
            "clicking the button should clear the awaiting marker"
        );
    }

    #[test]
    fn each_rate_button_computes_its_own_rate() {
        for (rate, expected) in [(1usize, 2usize), (2, 4), (3, 6)] {
            let mut world = World::new();
            world.init_resource::<LavaTheme>();
            let player = spawn_human_awaiting_with_cities(&mut world, 2);

            world.run_system_once(spawn_coinage_rate_ui).unwrap();
            let button = find_rate_button(&mut world, rate);
            world.trigger(Activate { entity: button });
            world.flush();

            let needs = world.get::<NeedsToPayTaxes>(player).unwrap();
            assert_eq!(needs.tokens_owed, expected, "rate {rate} on 2 cities");
        }
    }

    #[test]
    fn clicking_the_button_despawns_the_picker_via_cleanup() {
        let mut world = World::new();
        world.init_resource::<LavaTheme>();
        spawn_human_awaiting_with_cities(&mut world, 1);

        world.run_system_once(spawn_coinage_rate_ui).unwrap();
        let button = find_rate_button(&mut world, 1);
        world.trigger(Activate { entity: button });
        world.flush();
        world.run_system_once(cleanup_coinage_rate_ui).unwrap();

        assert!(
            world
                .query::<&CoinageRateUiRoot>()
                .iter(&world)
                .next()
                .is_none(),
            "picker should be despawned once the choice is resolved"
        );
    }
}
