use bevy::app::{App, Plugin};
use bevy::prelude::{in_state, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Query, Reflect, ResMut, Update};
use itertools::Itertools;
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::{BuiltCity, CityToken, CityTokenStock, Population, Stock, Treasury};
use crate::civilization::general::events::MoveTokensFromStockToAreaCommand;

pub struct CitySupportPlugin;

impl Plugin for CitySupportPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EliminateCity>()
            .add_event::<CheckPlayerCitySupport>()
            .add_event::<CheckCitySupportStatus>()
            .add_systems(OnEnter(GameActivity::CheckCitySupport), check_city_support)
            .add_systems(Update,
                         (
                             eliminate_city.run_if(in_state(GameActivity::CheckCitySupport)),
                             check_player_city_support.run_if(in_state(GameActivity::CheckCitySupport)),
                             check_city_support.run_if(in_state(GameActivity::CheckCitySupport)),
                             check_status.run_if(in_state(GameActivity::CheckCitySupport)),
                         ),
            )
        ;
    }
}

#[derive(Component, Debug, Reflect)]
pub struct HasTooManyCities {
    pub surplus_count: usize,
    pub needed_tokens: usize
}

#[derive(Event, Debug, Reflect)]
pub struct EliminateCity {
    pub city: Entity,
    pub area_entity: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckPlayerCitySupport {
    pub player: Entity,
}

#[derive(Event, Debug, Reflect)]
pub struct CheckCitySupportStatus;

pub fn eliminate_city(
    mut eliminate_city: EventReader<EliminateCity>,
    mut commands: Commands,
    mut city_token_stock: Query<&mut CityTokenStock>,
    area_population: Query<&mut Population>,
    city_token: Query<&CityToken>,
    mut move_tokens: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut check_player_city_support: EventWriter<CheckPlayerCitySupport>,
) {
    for eliminate in eliminate_city.read() {
        if let Ok(city_token) = city_token.get(eliminate.city) {
            if let Ok(mut city_stock) = city_token_stock.get_mut(city_token.player) {
                if let Ok(population) = area_population.get(eliminate.area_entity) {
                    move_tokens.send(MoveTokensFromStockToAreaCommand {
                        player_entity: city_token.player,
                        area_entity: eliminate.area_entity,
                        number_of_tokens: population.max_population,
                    });
                    commands.entity(eliminate.city).remove::<BuiltCity>();
                    city_stock.tokens.push(eliminate.city);
                    check_player_city_support.send(CheckPlayerCitySupport { player: city_token.player });
                }
            }
        }
    }
}

pub fn check_status(
    mut check_support_status: EventReader<CheckCitySupportStatus>,
    needs_city_support: Query<&HasTooManyCities>,
    mut next_state: ResMut<NextState<GameActivity>>
) {
    for _ in check_support_status.read() {
        if needs_city_support.is_empty() {
            next_state.set(GameActivity::PopulationExpansion);
        }
    }
}

pub fn check_player_city_support(
    mut check_player_city_support: EventReader<CheckPlayerCitySupport>,
    city_query: Query<&BuiltCity>,
    stock_query: Query<(&Stock, &Treasury)>,
    mut commands: Commands,
    mut check_status: EventWriter<CheckCitySupportStatus>
) {
    for check_player_support in check_player_city_support.read() {
        let cities = city_query
            .iter()
            .filter(|built_city| built_city.player == check_player_support.player)
            .map(|built_city| built_city.player)
            .collect::<Vec<_>>();

        let needed_population = cities.len() * 2;

        if let Ok((stock, treasury)) = stock_query.get(check_player_support.player) {
            let population_count = stock.max_tokens - stock.tokens.len() - treasury.tokens.len();
            if needed_population > population_count {
                let needed_tokens = needed_population - population_count;
                let surplus_count = (needed_population - population_count) / 2;
                commands.entity(check_player_support.player).insert(HasTooManyCities {
                    surplus_count,
                    needed_tokens,
                });
                println!("Player {:?} has {} too many cities", check_player_support.player, surplus_count);
            } else {
                commands.entity(check_player_support.player).remove::<HasTooManyCities>();
                check_status.send(CheckCitySupportStatus {});
            }
        }
    }
}

pub fn check_city_support(
    city_query: Query<&BuiltCity>,
    mut check_player_city_support: EventWriter<CheckPlayerCitySupport>,
    mut next_state: ResMut<NextState<GameActivity>>
) {
    if city_query.is_empty() {
        next_state.set(GameActivity::PopulationExpansion);
        return;
    }
    for (player, _) in city_query
        .iter()
        .chunk_by(|city| city.player).into_iter() {
        check_player_city_support.send(CheckPlayerCitySupport { player });
    };
}
