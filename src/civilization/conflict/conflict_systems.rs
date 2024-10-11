use crate::civilization::conflict::conflict_components::{UnresolvedCityConflict, UnresolvedConflict};
use crate::civilization::general::general_components::{BuiltCity, Population};
use crate::GameActivity;
use bevy::core::Name;
use bevy::prelude::{Commands, Entity, EventWriter, Has, NextState, Query, ResMut};
use bevy_console::PrintConsoleLine;

pub fn conflict_gate(
    conflicts: Query<&UnresolvedConflict>,
    city_conflicts: Query<&UnresolvedCityConflict>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if conflicts.is_empty() && city_conflicts.is_empty() {
        next_state.set(GameActivity::CheckCitySupport)
    }
}

pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population, Has<BuiltCity>)>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>
) {
    pop_query.iter().filter(|(_, _, pop, has_city)| {
        pop.is_conflict_zone(*has_city)
    }).for_each(|(conflict_zone, name, _, has_city)| {
        write_line.send(PrintConsoleLine::new(format!("Conflict zone found: {:?}", name)));
        if has_city {
            commands.entity(conflict_zone).insert(UnresolvedCityConflict);
        } else {
            commands.entity(conflict_zone).insert(UnresolvedConflict);
        }
    });
}
