use crate::civilization::conflict::conflict_components::{UnresolvedCityConflict, UnresolvedConflict};
use crate::civilization::general::general_components::{BuiltCity, Population};
use crate::GameActivity;
use bevy::core::Name;
use bevy::prelude::{debug, Commands, Entity, Has, NextState, Query, ResMut};

pub fn conflict_gate(
    conflicts: Query<&UnresolvedConflict>,
    city_conflicts: Query<&UnresolvedCityConflict>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if conflicts.is_empty() && city_conflicts.is_empty() {
        next_state.set(GameActivity::CityConstruction);
    }
}

pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population, Has<BuiltCity>)>,
    mut commands: Commands
) {
    pop_query.iter().filter(|(_, _, pop, has_city)| {
        pop.is_conflict_zone(*has_city)
    }).for_each(|(conflict_zone, name, _, has_city)| {
        if has_city {
            debug!("City Conflict Zone found: {:?}", name);
            commands.entity(conflict_zone).insert(UnresolvedCityConflict);
        } else {
            debug!("Conflict Zone found: {:?}", name);
            commands.entity(conflict_zone).insert(UnresolvedConflict);
        }
    });
}
