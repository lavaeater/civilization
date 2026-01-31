use crate::GameActivity;
use bevy::prelude::{Commands, Entity, Has, Name, NextState, Query, ResMut};
use crate::civilization::components::BuiltCity;
use crate::civilization::components::population::Population;
use crate::civilization::concepts::conflict::conflict_components::{UnresolvedCityConflict, UnresolvedConflict};

pub fn conflict_gate(
    conflicts: Query<&UnresolvedConflict>,
    city_conflicts: Query<&UnresolvedCityConflict>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if conflicts.is_empty() && city_conflicts.is_empty() {
        //debug!("No conflicts found, moving to next state");
        next_state.set(GameActivity::CityConstruction);
    }
}

pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population, Has<BuiltCity>)>,
    mut commands: Commands,
) {
    pop_query
        .iter()
        .filter(|(_, _, pop, has_city)| pop.is_conflict_zone(*has_city))
        .for_each(|(conflict_zone, _name, _, has_city)| {
            if has_city {
                //debug!("City Conflict Zone found: {:#?}", name);
                commands
                    .entity(conflict_zone)
                    .insert(UnresolvedCityConflict);
            } else {
                //debug!("Conflict Zone found: {:#?}", name);
                commands.entity(conflict_zone).insert(UnresolvedConflict);
            }
        });
}
