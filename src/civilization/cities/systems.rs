use bevy::prelude::{Commands, Entity, Query, With, Without};
use crate::civilization::cities::components::CityBuildTargets;
use crate::civilization::general::components::{BuiltCity, CitySite, Population};
use crate::player::Player;

pub fn setup_players_and_cities(
    player_query: Query<Entity, With<Player>>,
    query: Query<(Entity, &Population, &CitySite), Without<BuiltCity>>,
    mut commands: Commands
) {
    for player_entity in player_query.iter() {
        let targets = query.iter().filter(|(_, population, _)| {
            return if population.player_tokens.contains_key(&player_entity) {
                let tokens = population.player_tokens.get(&player_entity).unwrap();
                tokens.len() >= 6
            } else {
                false
            }
        }).map(|(entity, _, _)| entity).collect();

        commands.entity(player_entity)
            .insert(CityBuildTargets {
                targets
            });
    }
}