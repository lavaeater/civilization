use bevy::prelude::{Children, Component, Entity, EventReader, EventWriter, Parent, Query, Reflect, ResMut, Resource, With};
use bevy::utils::HashMap;
use crate::civilization::civ::{GameActivity, GameActivityEnded, GameActivityStarted, Stock};
use crate::player::Player;

#[derive(Component, Debug, Reflect)]
pub struct Census {
    pub population: usize,
}

#[derive(Resource, Debug, Reflect)]
pub struct CensusOrder {
    pub players_by_population: Vec<Entity>,
}

pub fn perform_census(
    mut start_activity: EventReader<GameActivityStarted>,
    mut end_activity: EventWriter<GameActivityEnded>,
    stock_query: Query<(&Parent, &Children, &Stock)>,
    mut player_query: Query<(&mut Census, Entity), With<Player>>,
    mut census_order: ResMut<CensusOrder>,
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Census {
            for (parent, tokens, stock) in stock_query.iter() {
                if let Ok((mut census, _)) = player_query.get_mut(parent.get()) {
                    census.population = stock.max_tokens - tokens.iter().count();
                }
            }
            census_order.players_by_population.clear();
            let mut hash_to_sort = HashMap::new();
            for (census, entity) in player_query.iter_mut() {
                hash_to_sort.insert(entity, census.population);
            }
            let mut ordered: Vec<(Entity, usize)> = hash_to_sort.into_iter().collect();
            ordered.sort_by(|a, b| b.1.cmp(&a.1));

            census_order.players_by_population = ordered.into_iter().map(|(entity, _)| entity).collect();
            end_activity.send(GameActivityEnded(GameActivity::Census));
        }
    }
}