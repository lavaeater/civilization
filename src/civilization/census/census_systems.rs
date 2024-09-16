use crate::civilization::census::census_components::{Census, HasPopulation};
use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::general::general_components::{Population, PlayerStock, Treasury};
use bevy::prelude::{Commands, Entity, NextState, Query, ResMut};
use bevy::utils::HashMap;
use crate::GameActivity;
/***
Checks and marks areas / populations with HasPopulation to
simplify queries later. This is normal
 */
pub fn check_areas_for_population(
    mut area_query: Query<(Entity, &mut Population)>,
    mut commands: Commands,
) {
    for (area, population) in area_query.iter_mut() {
        if population.has_population() {
            commands.entity(area).insert(HasPopulation {});
        } else {
            commands.entity(area).remove::<HasPopulation>();
        }
    }
}

pub fn perform_census(
    mut stock_query: Query<(Entity, &PlayerStock, &Treasury, &mut Census)>,
    mut census_order: ResMut<GameInfoAndStuff>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    census_order.census_order.clear();
    let mut hash_to_sort = HashMap::new();
    for (player, stock, treasury, mut census) in stock_query.iter_mut() {
        census.population = stock.max_tokens - stock.tokens.len() - treasury.tokens.len();
        hash_to_sort.insert(player, census.population);
    }
    let mut ordered: Vec<(Entity, usize)> = hash_to_sort.into_iter().collect();
    ordered.sort_by(|a, b| b.1.cmp(&a.1));

    census_order.census_order = ordered.into_iter().map(|(entity, _)| entity).collect();
    next_state.set(GameActivity::Movement);
}

