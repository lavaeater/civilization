use bevy::prelude::{in_state, Children, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Parent, Plugin, Query, Reflect, ResMut, Resource, With};
use bevy::utils::HashMap;
use crate::civilization::civ::{Area, Population, Stock};
use crate::civilization::game_phases::{GameActivity, GameActivityEnded, GameActivityStarted};
use crate::GameState;
use crate::player::Player;

pub struct CensusPlugin;

impl Plugin for CensusPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_event::<CensusEnded>()
            .add_event::<CheckAreasForPopulationCommand>()
            .add_systems(
                bevy::app::Update, (
                    perform_census
                        .run_if(in_state(GameState::Playing)),
                    check_areas_for_population
                        .run_if(in_state(GameState::Playing)),
                    end_census
                        .run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug)]
pub struct CensusEnded;

#[derive(Event, Debug)]
pub struct CheckAreasForPopulationCommand;

#[derive(Component, Debug, Reflect)]
pub struct Census {
    pub population: usize,
}

#[derive(Component, Debug, Reflect)]
pub struct HasPopulation;

#[derive(Resource, Debug, Reflect, Default)]
pub struct GameInfoAndStuff {
    pub census_order: Vec<Entity>,
    pub left_to_move: Vec<Entity>,
    pub current_mover: Option<Entity>,
}

pub fn end_census(
    mut census_ended: EventReader<CensusEnded>,
    mut game_activity_ended: EventWriter<GameActivityEnded>,
) {
    for _ in census_ended.read() {
        game_activity_ended.send(GameActivityEnded(GameActivity::Census));
    }
}

/***
Checks and marks areas / populations with HasPopulation to
simplify queries later. This is normal
 */
pub fn check_areas_for_population(
    mut start_checking: EventReader<CheckAreasForPopulationCommand>,
    area_query: Query<(Entity, &Children), With<Area>>,
    population_query: Query<(Entity, &Children), With<Population>>,
    mut commands: Commands,
    mut end_census: EventWriter<CensusEnded>,
) {
    for _ in start_checking.read() {
        for (area, children) in area_query.iter() {
            for area_child in children {
                if let Ok((pop_ent, tokens)) = population_query.get(*area_child) {
                    if tokens.iter().count() > 0 {
                        commands.entity(area).insert(HasPopulation {});
                        commands.entity(pop_ent).insert(HasPopulation {});
                    } else {
                        commands.entity(area).remove::<HasPopulation>();
                        commands.entity(pop_ent).remove::<HasPopulation>();
                    }
                }
            }
        }
        end_census.send(CensusEnded {});
    }
}

pub fn perform_census(
    mut start_activity: EventReader<GameActivityStarted>,
    mut check_areas_for_population_command: EventWriter<CheckAreasForPopulationCommand>,
    stock_query: Query<(&Parent, &Children, &Stock)>,
    mut player_query: Query<(&mut Census, Entity), With<Player>>,
    mut census_order: ResMut<GameInfoAndStuff>,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Census {
            for (parent, tokens, stock) in stock_query.iter() {
                if let Ok((mut census, _)) = player_query.get_mut(parent.get()) {
                    census.population = stock.max_tokens - tokens.iter().count();
                }
            }
            census_order.census_order.clear();
            let mut hash_to_sort = HashMap::new();
            for (census, entity) in player_query.iter_mut() {
                hash_to_sort.insert(entity, census.population);
            }
            let mut ordered: Vec<(Entity, usize)> = hash_to_sort.into_iter().collect();
            ordered.sort_by(|a, b| b.1.cmp(&a.1));

            census_order.census_order = ordered.into_iter().map(|(entity, _)| entity).collect();
            check_areas_for_population_command.send(CheckAreasForPopulationCommand {});
        }
    }
}

