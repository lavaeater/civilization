use crate::civilization::components::{BuiltCity, Population};
use crate::civilization::concepts::conflict::conflict_components::{
    UnresolvedCityConflict, UnresolvedConflict,
};
use crate::civilization::ConflictCounterResource;
use crate::GameActivity;
use bevy::prelude::{info, Commands, Entity, Has, Name, NextState, Query, ResMut};

pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population, Has<BuiltCity>)>,
    mut commands: Commands,
    mut conflict_counter_resource: ResMut<ConflictCounterResource>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    conflict_counter_resource.0 = 0; // reset counter
    pop_query
        .iter()
        .filter(|(_, _, pop, has_city)| pop.is_conflict_zone(*has_city))
        .for_each(|(conflict_zone, _name, _, has_city)| {
            conflict_counter_resource.0 += 1;
            if has_city {
                commands
                    .entity(conflict_zone)
                    .insert(UnresolvedCityConflict);
            } else {
                commands.entity(conflict_zone).insert(UnresolvedConflict);
            }
        });
    if conflict_counter_resource.0 == 0 {
        info!("No conflicts found, transitioning to CityConstruction");
        next_state.set(GameActivity::CityConstruction);
    } else {
        info!(
            "Found {} conflicts, remaining in Conflict",
            conflict_counter_resource.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::components::*;
    use crate::civilization::concepts::conflict::conflict_triggers::*;
    use crate::civilization::CameraFocusQueue;
    use crate::GameState;
    use bevy::ecs::system::{RunSystemError, RunSystemOnce};
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.insert_state(GameState::Playing);
        app.add_sub_state::<GameActivity>();
        app.insert_state(GameActivity::Conflict);
        app.init_resource::<ConflictCounterResource>();
        app.init_resource::<CameraFocusQueue>();
        app.add_observer(on_add_unresolved_conflict);
        app.add_observer(on_add_unresolved_city_conflict);

        app
    }

    fn transition_to_conflict(app: &mut App) -> std::result::Result<(), RunSystemError> {
        app.world_mut().run_system_once(find_conflict_zones)
    }

    /// Helper: spawn an area with a Name, Population, and optional BuiltCity.
    fn spawn_area(
        world: &mut World,
        name: &str,
        max_pop: usize,
        player_token_counts: &[(Entity, usize)],
        built_city: Option<BuiltCity>,
    ) -> Entity {
        let mut pop = Population::new(max_pop);
        for &(player, count) in player_token_counts {
            for _ in 0..count {
                let token = world.spawn_empty().id();
                pop.add_token_to_area(player, token);
            }
        }
        let mut entity_commands = world.spawn((Name::new(name.to_string()), pop, Transform::default()));
        if let Some(city) = built_city {
            entity_commands.insert(city);
        }
        entity_commands.id()
    }

    // ========================================================================
    // Integration: find_conflict_zones → observer → counter-based transition
    // ========================================================================

    #[test]
    fn test_no_conflicts_transitions_immediately() {
        let mut app = create_test_app();
        let p1 = app.world_mut().spawn_empty().id();
        // Area with 1 player, under max → no conflict
        spawn_area(app.world_mut(), "Safe Area", 4, &[(p1, 2)], None);

        // Run updates: first runs find_conflict_zones which sets NextState,
        // second processes the state transition
        transition_to_conflict(&mut app).unwrap();
        app.update();
        app.update();

        let state = app.world().resource::<State<GameActivity>>();
        assert_eq!(
            *state.get(),
            GameActivity::CityConstruction,
            "Should transition to CityConstruction when no conflicts exist"
        );
    }

    #[test]
    fn test_regular_conflict_resolves_and_transitions() {
        let mut app = create_test_app();
        let p1 = app.world_mut().spawn_empty().id();
        let p2 = app.world_mut().spawn_empty().id();
        // Area max_pop=2, p1 has 2, p2 has 2 → conflict (total 4 > max 2)
        spawn_area(
            app.world_mut(),
            "Conflict Area",
            2,
            &[(p1, 2), (p2, 2)],
            None,
        );

        // Transition into Conflict state to trigger OnEnter, then run frames for resolution
        transition_to_conflict(&mut app).unwrap();
        app.update();
        app.update();

        let state = app.world().resource::<State<GameActivity>>();
        assert_eq!(
            *state.get(),
            GameActivity::CityConstruction,
            "Should transition after conflict is resolved"
        );

        // Verify no UnresolvedConflict components remain
        let conflicts = app
            .world_mut()
            .query::<&UnresolvedConflict>()
            .iter(app.world())
            .count();
        assert_eq!(conflicts, 0, "All UnresolvedConflict should be removed");
    }

    #[test]
    fn test_multiple_conflict_zones_all_resolve() {
        let mut app = create_test_app();
        let p1 = app.world_mut().spawn_empty().id();
        let p2 = app.world_mut().spawn_empty().id();
        let p3 = app.world_mut().spawn_empty().id();

        // Multiple conflict zones
        spawn_area(app.world_mut(), "Area A", 2, &[(p1, 3), (p2, 1)], None);
        spawn_area(app.world_mut(), "Area B", 3, &[(p2, 3), (p3, 2)], None);
        spawn_area(app.world_mut(), "Area C", 1, &[(p1, 1), (p3, 1)], None);

        transition_to_conflict(&mut app).unwrap();
        for _ in 0..5 {
            app.update();
        }

        let state = app.world().resource::<State<GameActivity>>();
        assert_eq!(
            *state.get(),
            GameActivity::CityConstruction,
            "Should transition after all conflicts are resolved"
        );

        let conflicts = app
            .world_mut()
            .query::<&UnresolvedConflict>()
            .iter(app.world())
            .count();
        let city_conflicts = app
            .world_mut()
            .query::<&UnresolvedCityConflict>()
            .iter(app.world())
            .count();
        assert_eq!(conflicts, 0);
        assert_eq!(city_conflicts, 0);
    }

    #[test]
    fn test_city_conflict_with_small_invader_resolves() {
        // City owner has tokens, one other player with <= 6 tokens → tokens eliminated
        let mut app = create_test_app();
        let city_owner = app.world_mut().spawn_empty().id();
        let invader = app.world_mut().spawn_empty().id();

        // The invader needs TokenStock and PlayerAreas for return_all_tokens_from_area_to_player
        app.world_mut().entity_mut(invader).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));
        // City owner also needs these components for the query
        app.world_mut().entity_mut(city_owner).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));

        let city = BuiltCity::new(Entity::PLACEHOLDER, city_owner);
        spawn_area(
            app.world_mut(),
            "City Area",
            4,
            &[(city_owner, 2), (invader, 3)],
            Some(city),
        );

        transition_to_conflict(&mut app).unwrap();
        for _ in 0..5 {
            app.update();
        }

        let state = app.world().resource::<State<GameActivity>>();
        assert_eq!(
            *state.get(),
            GameActivity::CityConstruction,
            "City conflict with small invader should resolve and transition"
        );

        let city_conflicts = app
            .world_mut()
            .query::<&UnresolvedCityConflict>()
            .iter(app.world())
            .count();
        assert_eq!(
            city_conflicts, 0,
            "UnresolvedCityConflict should be removed"
        );
    }

    #[test]
    fn test_city_conflict_with_large_invader_resolves() {
        // City owner has tokens, one other player with > 6 tokens → city replaced, regular conflict
        let mut app = create_test_app();
        let city_owner = app.world_mut().spawn_empty().id();
        let invader = app.world_mut().spawn_empty().id();

        // Both need full component sets
        app.world_mut().entity_mut(city_owner).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));
        app.world_mut().entity_mut(invader).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));

        let city = BuiltCity::new(Entity::PLACEHOLDER, city_owner);
        spawn_area(
            app.world_mut(),
            "City Area",
            4,
            &[(city_owner, 2), (invader, 7)],
            Some(city),
        );

        transition_to_conflict(&mut app).unwrap();
        for _ in 0..5 {
            app.update();
        }

        let state = app.world().resource::<State<GameActivity>>();
        assert_eq!(
            *state.get(),
            GameActivity::CityConstruction,
            "City conflict with large invader should resolve and transition"
        );

        let conflicts = app
            .world_mut()
            .query::<&UnresolvedConflict>()
            .iter(app.world())
            .count();
        let city_conflicts = app
            .world_mut()
            .query::<&UnresolvedCityConflict>()
            .iter(app.world())
            .count();
        assert_eq!(conflicts, 0);
        assert_eq!(city_conflicts, 0);
    }

    #[test]
    fn test_city_conflict_large_invader_no_counter_underflow() {
        // Regression test: when a city conflict chains into a regular conflict
        // (large invader path), the counter must only be decremented once — not
        // once in on_add_unresolved_city_conflict AND again in on_add_unresolved_conflict.
        // Before the fix this caused "attempt to subtract with overflow" panic.
        let mut app = create_test_app();
        let city_owner = app.world_mut().spawn_empty().id();
        let invader = app.world_mut().spawn_empty().id();

        app.world_mut().entity_mut(city_owner).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));
        app.world_mut().entity_mut(invader).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));

        let city = BuiltCity::new(Entity::PLACEHOLDER, city_owner);
        // Single city conflict zone: invader has >6 tokens → large invader path
        // chains UnresolvedCityConflict → UnresolvedConflict
        spawn_area(
            app.world_mut(),
            "City Area",
            4,
            &[(city_owner, 2), (invader, 7)],
            Some(city),
        );

        // This must not panic with "attempt to subtract with overflow".
        // Before the fix, the city conflict observer decremented the counter,
        // then chained into the regular conflict observer which decremented again,
        // causing a usize underflow panic.
        transition_to_conflict(&mut app).unwrap();
        for _ in 0..10 {
            app.update();
        }

        // The counter must not have underflowed
        let counter = app.world().resource::<ConflictCounterResource>();
        assert_eq!(
            counter.0, 0,
            "Counter should be exactly 0 after all conflicts resolve — not underflowed"
        );
    }

    #[test]
    fn test_city_conflict_owner_only_no_hang() {
        // City with only the owner's tokens → is_conflict_zone returns true
        // but on_add_unresolved_city_conflict should still remove the component
        let mut app = create_test_app();
        let city_owner = app.world_mut().spawn_empty().id();

        app.world_mut().entity_mut(city_owner).insert((
            TokenStock::new(47, vec![]),
            PlayerAreas::default(),
            CityTokenStock::new(7, vec![]),
            PlayerCities::default(),
        ));

        let city = BuiltCity::new(Entity::PLACEHOLDER, city_owner);
        spawn_area(
            app.world_mut(),
            "City Area",
            4,
            &[(city_owner, 2)],
            Some(city),
        );

        transition_to_conflict(&mut app).unwrap();
        for _ in 0..5 {
            app.update();
        }

        let state = app.world().resource::<State<GameActivity>>();
        assert_eq!(
            *state.get(),
            GameActivity::CityConstruction,
            "City with only owner should not hang — should transition"
        );
    }
}
