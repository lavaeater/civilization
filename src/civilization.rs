use bevy::prelude::{Component, Entity, Event, EventReader, Query, With};
use crate::player::Player;

#[derive(Event, Debug)]
struct BeginPopulationExpansion;

#[derive(Event, Debug)]
struct MoveTokensFromStockToArea {
    pub area_entity: Entity,
    pub player_entity: Entity,
    pub number_of_tokens: u8
}

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
    area_query: Query<&Area, With<AreaHasPopulation>>,
    player_query: Query<&Player>
) {

}

fn move_tokens_from_stock_to_area(
    mut move_event: EventReader<MoveTokensFromStockToArea>,
) {
    
}

fn remove_surplus_population(query: Query<&Area, With<AreaHasSurplusPopulation>>) {

}