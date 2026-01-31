use crate::civilization::components::*;
use crate::civilization::concepts::city_construction::city_construction_components::*;
use crate::civilization::concepts::city_construction::city_construction_events::*;
use crate::civilization::concepts::map::map_plugin::AvailableFactions;
use crate::civilization::functions::{build_city_in_area, return_all_tokens_from_area_to_players};
use crate::civilization::game_moves::game_moves_components::AvailableMoves;
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{
    Commands, Entity, MessageReader, MessageWriter, NextState, Query, Res, ResMut, Transform, With,
};
use crate::civilization::components::population::Population;

pub fn city_building_gate(
    query: Query<&IsBuilding>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if query.is_empty() {
        //debug!("No one is building cities, moving on!");
        next_state.set(GameActivity::RemoveSurplusPopulation);
    }
}

pub fn build_city(
    mut command: MessageReader<BuildCityCommand>,
    mut city_population: Query<(&mut Population, &Transform)>,
    mut player_query: Query<(
        &mut CityTokenStock,
        &mut PlayerAreas,
        &mut PlayerCities,
        &mut TokenStock,
        &Faction,
    )>,
    mut commands: Commands,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
    game_factions: Res<AvailableFactions>,
) {
    for build_city in command.read() {
        if let Ok((mut population, _)) = city_population.get_mut(build_city.area) {
            return_all_tokens_from_area_to_players(&mut population, &mut commands);
        }

        if let Ok((mut city_stock, _, mut player_cities, _, faction)) =
            player_query.get_mut(build_city.player)
        {
            if let Ok((_, area_transform)) = city_population.get_mut(build_city.area) {
                let texture = game_factions
                    .faction_city_icons
                    .get(&faction.faction)
                    .unwrap()
                    .clone();
                build_city_in_area(
                    &mut commands,
                    texture,
                    build_city,
                    &mut city_stock,
                    &mut player_cities,
                    area_transform,
                );
                recalculate_player_moves.write(RecalculatePlayerMoves::new(build_city.player));
            }
        }
    }
}

pub fn on_enter_city_construction(
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for player_entity in player_query.iter() {
        commands.entity(player_entity).insert(IsBuilding);
    }
}

pub fn end_player_city_construction(
    mut end_activity: MessageReader<EndPlayerCityConstruction>,
    mut commands: Commands,
) {
    for event in end_activity.read() {
        commands.entity(event.player).remove::<IsBuilding>();
        commands.entity(event.player).remove::<AvailableMoves>();
    }
}
