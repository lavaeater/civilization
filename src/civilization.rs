use bevy::prelude::{Component, Event, Query, With};

#[derive(Event)]
struct BeginPopulationExpansion;

#[derive(Component)]
struct Area {
    pub max_population: u8
}

/***
Is it reasonable to have a marker component for this or will that add complexity somewhere
else in the game? I don't knooow
 */
#[derive(Component)]
struct AreaHasPopulation;

fn expand_ppopulation(query: Query<&Area, With<AreaHasPopulation>>) {
    
}