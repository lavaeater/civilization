use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Component, Entity, Event, EventReader, OnEnter, Query, Reflect};
use itertools::Itertools;
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::{BuiltCity, Population, Stock, Treasury};

pub struct CitySupportPlugin;

impl Plugin for CitySupportPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EliminateCity>()
            .add_systems(OnEnter(GameActivity::CheckCitySupport), check_city_support)
        ;
    }
}

#[derive(Component, Debug, Reflect)]
pub struct HasTooManyCities;
#[derive(Event, Debug, Reflect)]
pub struct EliminateCity {
    pub city: Entity,
    pub area_entity: Entity
}


fn eliminate_city(
    mut eliminate_city: EventReader<EliminateCity>,

) {
    for eliminate in eliminate_city.read() {

    }
}

fn check_city_support(
    city_query: Query<&BuiltCity>,
    stock_query: Query<(&Stock, &Treasury)>,
    mut commands: Commands
) {
    for (player, cities) in city_query
        .iter()
        .chunk_by(|city| city.player).into_iter() {
            let needed_population = cities.count() * 2;
            let (stock, treasury) = stock_query.get(player).unwrap();
            let population_count = stock.max_tokens - stock.tokens.len() - treasury.tokens.len();
            if needed_population > population_count {
                let surplus_count = (needed_population - population_count) / 2;
                commands.entity(player).insert(HasTooManyCities {});
                println!("Player {:?} has {} too many cities", player, surplus_count);
            }
        };
}
