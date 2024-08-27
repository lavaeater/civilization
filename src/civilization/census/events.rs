use bevy::prelude::Event;

#[derive(Event, Debug)]
pub struct CensusEnded;

#[derive(Event, Debug)]
pub struct CheckAreasForPopulationCommand;
