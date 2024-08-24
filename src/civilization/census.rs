use bevy::prelude::{Children, Component, EventReader, EventWriter, Parent, Query, Reflect, With};
use crate::civilization::civ::{GameActivity, GameActivityEnded, GameActivityStarted, Stock};
use crate::player::Player;

#[derive(Component, Debug, Reflect)]
pub struct Census {
    pub population: usize
}

pub fn perform_census(
    mut start_activity: EventReader<GameActivityStarted>,
    mut end_activity: EventWriter<GameActivityEnded>,
    stock_query: Query<(&Parent, &Children), With<Stock>>,
    mut player_query: Query<&mut Census, With<Player>>
) {
    for activity in start_activity.read() {
        if activity.0 == GameActivity::Census {
            for (parent, tokens) in stock_query.iter() {
                if let Ok(mut census) = player_query.get_mut(parent.get()) {
                    census.population = tokens.iter().count();
                }
            }
            end_activity.send(GameActivityEnded(GameActivity::Census));
        }
    }
}