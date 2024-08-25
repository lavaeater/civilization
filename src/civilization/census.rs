use bevy::prelude::{Children, Commands, Component, Entity, EventReader, EventWriter, Parent, Query, Reflect, ResMut, Resource, With};
use bevy::utils::HashMap;
use crate::civilization::civ::{Area, GameActivity, GameActivityEnded, GameActivityStarted, Population, Stock};
use crate::player::Player;

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

/***
Checks and marks areas / populations with HasPopulation to
simplify queries later. This is normal
 */
pub fn check_areas_for_population(
    mut start_activity: EventReader<GameActivityStarted>,
    area_query: Query<(Entity, &Children), With<Area>>,
    population_query: Query<(Entity, &Children), With<Population>>,
    mut commands: Commands,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Census {
            for (area, children) in area_query.iter() {
                if let Some((pop_ent, tokens)) = population_query.get(area) {
                    if tokens.into_iter().any() {
                        commands.entity(area).insert(HasPopulation {});
                        commands.entity(pop_ent).insert(HasPopulation {});
                    } else {
                        commands.entity(area).remove::<HasPopulation>();
                        commands.entity(pop_ent).remove::<HasPopulation>();
                    }
                }
            }
        }
    }
}

pub fn perform_census(
    mut start_activity: EventReader<GameActivityStarted>,
    mut end_activity: EventWriter<GameActivityEnded>,
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
            end_activity.send(GameActivityEnded(GameActivity::Census));
        }
    }
}