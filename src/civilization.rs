use bevy::prelude::{BuildChildren, Children, Commands, Component, Entity, Event, EventReader, Query, With};
use bevy_console::Command;
use crate::player::Player;

#[derive(Event, Debug)]
struct BeginPopulationExpansion;

#[derive(Event, Debug)]
struct MoveTokensFromStockToArea {
    pub area_entity: Entity,
    pub player_entity: Entity,
    pub number_of_tokens: u8,
}

#[derive(Event, Debug)]
struct MoveTokenFromAreaToArea {
    pub from_area: Entity,
    pub to_area: Entity,
    pub tokens: Vec<Entity>,
}

#[derive(Event, Debug)]
struct CheckAreaSurplus {
    pub area: Entity,
}

#[derive(Component, Debug)]
struct Area {
    pub max_population: u8,
}

/***
Is it reasonable to have a marker component for this or will that add complexity somewhere
else in the game? I don't knooow
 */
#[derive(Component, Debug)]
struct AreaHasPopulation;

#[derive(Component, Debug)]
struct AreaHasSurplusPopulation;

fn move_token_from_area_to_area(
    mut move_events: EventReader<MoveTokenFromAreaToArea>,
    mut commands: Commands,
) {
    for ev in move_events.read() {
        commands.entity(ev.from_area).remove_children(&ev.tokens);
        commands.entity(ev.to_area).push_children(&ev.tokens);
    }
}

fn expand_population(
    mut begin_event: EventReader<BeginPopulationExpansion>,
    area_query: Query<&Area, With<AreaHasPopulation>>,
    player_query: Query<&Player>,
) {
    /*
    q_parent: Query<(&MySquadDamage, &Children)>,
        q_child: Query<&MyUnitHealth>,
    ) {
        // get the properties of each squad
        for (squad_dmg, children) in q_parent.iter() {
            // `children` is a collection of Entity IDs
            for &child in children.iter() {
                // get the health of each child unit
                let health = q_child.get(child);

                // do something
     */
}

fn move_tokens_from_stock_to_area(
    mut move_event: EventReader<MoveTokensFromStockToArea>,
) {}

fn remove_surplus_population(query: Query<&Area, With<AreaHasSurplusPopulation>>) {}