use bevy::prelude::{Component, Event, EventReader, Query, With};

#[derive(Event, Debug)]
struct BeginPopulationExpansion;

#[derive(Component, Debug)]
struct Area {
    pub max_population: u8
}

/***
Is it reasonable to have a marker component for this or will that add complexity somewhere
else in the game? I don't knooow
 */
#[derive(Component, Debug)]
struct AreaHasPopulation;

#[derive(Component, Debug)]
struct AreaHasSurplusPopulation;

fn expand_ppopulation(
    mut begin_event: EventReader<BeginPopulationExpansion>,
    query: Query<&Area, With<AreaHasPopulation>>) {

}

fn remove_surplus_population(query: Query<&Area, With<AreaHasSurplusPopulation>>)