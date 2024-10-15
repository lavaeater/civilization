use crate::civilization::city_construction::prelude::*;
use crate::civilization::game_moves::prelude::*;
use crate::civilization::general::prelude::*;
use crate::civilization::map::map_plugin::AvailableFactions;
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{debug, default, Commands, Entity, EventReader, EventWriter, Handle, Image, Mut, NextState, Query, Res, ResMut, SpriteBundle, Transform, Vec3, With};

pub fn city_building_gate(
    query: Query<&IsBuilding>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if query.is_empty() {
        debug!("No one is building cities, moving on!");
        next_state.set(GameActivity::RemoveSurplusPopulation);
    }
}

pub fn build_city(
    mut command: EventReader<BuildCityCommand>,
    mut city_population: Query<(&mut Population, &Transform)>,
    mut player_query: Query<(&mut CityTokenStock, &mut PlayerAreas, &mut PlayerCities, &mut TokenStock, &Faction)>,
    mut commands: Commands,
    mut recalculate_player_moves: EventWriter<RecalculatePlayerMoves>,
    game_factions: Res<AvailableFactions>,
) {
    for build_city in command.read() {
        if let Ok((mut population, area_transform)) = city_population.get_mut(build_city.area) {
            return_all_tokens_from_area_to_players(build_city.area,
                                                   &mut population, 
                                                   &mut player_query.transmute_lens::<(&mut TokenStock, &mut PlayerAreas)>().query());
        }
                
        if let Ok(
            (mut city_stock, _, mut player_cities, _, faction)) = player_query.get_mut(build_city.player) {
            if let Ok((_, area_transform)) = city_population.get_mut(build_city.area) {
                let texture = game_factions.faction_city_icons.get(&faction.faction).unwrap().clone();
                build_city_in_area(&mut commands, texture, build_city, &mut city_stock, &mut player_cities, area_transform);
                recalculate_player_moves.send(RecalculatePlayerMoves::new(build_city.player));
            }
        }
    }
}

pub fn on_enter_city_construction(
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for player_entity in player_query.iter() {
        commands.entity(player_entity)
            .insert(IsBuilding);
    }
}

pub fn end_player_city_construction(
    mut end_activity: EventReader<EndPlayerCityConstruction>,
    mut commands: Commands,
) {
    for event in end_activity.read() {
        commands.entity(event.player)
            .remove::<IsBuilding>();
        commands.entity(event.player)
            .remove::<AvailableMoves>();
    }
}
