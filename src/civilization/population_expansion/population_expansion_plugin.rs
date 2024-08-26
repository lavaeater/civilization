use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Query, Reflect, ResMut, With, Without};
use bevy::hierarchy::Children;
use bevy::log::info;
use bevy::utils::HashMap;
use itertools::Itertools;
use winit::event::Event::NewEvents;
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::plugin::{Population, Stock, Token, MoveTokensFromStockToAreaCommand};
use crate::GameState;
use crate::player::Player;

pub struct PopulationExpansionPlugin;

impl Plugin for PopulationExpansionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PopulationExpansionEnded>()
            .add_event::<BeginPopulationExpansionEvent>()
            .add_event::<StartManualPopulationExpansionEvent>()
            .add_systems(OnEnter::<GameActivity::PopulationExpansion>,
                         check_population_expansion_eligibility)
            .add_systems(
                Update, (
                    expand_population
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                    end_population_expansion
                        .run_if(in_state(GameActivity::PopulationExpansion)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug)]
pub struct BeginPopulationExpansionEvent;

#[derive(Event, Debug)]
pub struct StartManualPopulationExpansionEvent;

#[derive(Event, Debug)]
pub struct PopulationExpansionEnded;

#[derive(Component, Debug)]
pub struct ExpandManually;

#[derive(Component, Debug, Reflect)]
pub struct ExpandAutomatically {
    pub required_tokens: usize,
}

#[derive(Component, Debug)]
pub struct NeedsExpansion;

pub fn end_population_expansion(
    mut population_expansion_ended: EventReader<PopulationExpansionEnded>,
) {
    for _ in population_expansion_ended.read() {}
}

/***
    * This system checks if the player has enough tokens to expand the population in all areas.
 */
pub fn check_population_expansion_eligibility(
    area_pop_query: Query<(Entity, &Population), Without<NeedsExpansion>>,
    player_stock_query: Query<(Entity, &Stock)>,
    mut commands: Commands,
) {
    for (player, stock) in player_stock_query.iter() {
        // how many tokens has the player?
        let tokens_in_stock = stock.iter().count();

        let mut required_tokens = 0;
        for (pop_entity, pop) in area_pop_query.iter() {
            if let Some(p) = pop.population.get(player) {
                let rt = match *p.len() {
                    1 => { 1 }
                    0 => { 0 }
                    _ => { 2 }
                };
                if rt > 0 {
                    commands.entity(pop_entity).insert(NeedsExpansion{});
                }

                required_tokens +=
            };
        }
        if required_tokens <= tokens_in_stock {
            commands
                .entity(player)
                .insert(ExpandAutomatically { required_tokens });
        } else {
            commands
                .entity(player)
                .insert(ExpandManually {});
        }
    }
}

/*
Later, bro
 */
// pub fn handle_manual_population_expansion(
//     mut start_reader: EventReader<StartManualPopulationExpansionEvent>,
//     mut expand_writer: EventWriter<BeginPopulationExpansionEvent>,
// ) {
//     for _start in start_reader.read() {
//         expand_writer.send(BeginPopulationExpansionEvent {});
//     }
// }

pub fn expand_population(
    area_query: Query<(Entity, &Population)>,
    mut to_expand: Query<&mut ExpandAutomatically>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>,

) {
    for (pop_entity, pop) in area_query.iter() {
        for (player, tokens) in pop.population.iter() {
            if to_expand.contains(*player) {
                if tokens.len() == 1 {
                    event_writer.send(MoveTokensFromStockToAreaCommand {
                        population_entity: pop_entity,
                        stock_entity: *player,
                        number_of_tokens: 1,
                    });
                } else if tokens.len() > 1 {
                    event_writer.send(MoveTokensFromStockToAreaCommand {
                        population_entity: pop_entity,
                        stock_entity: *player,
                        number_of_tokens: 2,
                    });
                }
            }
        }
    }
}