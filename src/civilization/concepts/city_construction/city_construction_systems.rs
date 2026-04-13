use crate::GameActivity;
use crate::civilization::CivCardName;
use crate::civilization::components::*;
use crate::civilization::concepts::city_construction::city_construction_components::{
    CityConstructionPhaseActive, IsBuilding,
};
use crate::civilization::concepts::city_construction::city_construction_events::*;
use crate::civilization::concepts::civ_cards::PlayerCivilizationCards;
use crate::civilization::concepts::map::map_plugin::AvailableFactions;
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::civilization::functions::{build_city_in_area, return_all_tokens_from_area_to_players};
use crate::civilization::game_moves::{AvailableMoves, RecalculatePlayerMoves};
use crate::player::Player;
use bevy::prelude::{
    Commands, Entity, MessageReader, MessageWriter, NextState, Query, Res, ResMut, Transform, With,
    info,
};

pub fn city_building_gate(
    query: Query<&IsBuilding>,
    phase_active: Option<Res<CityConstructionPhaseActive>>,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut commands: Commands,
) {
    if phase_active.is_none() {
        return;
    }

    let building_count = query.iter().count();
    if building_count == 0 {
        info!(
            "[CITY_CONSTRUCTION] Gate: No players with IsBuilding, transitioning to RemoveSurplusPopulation"
        );
        commands.remove_resource::<CityConstructionPhaseActive>();
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
        &mut Treasury,
        Option<&PlayerCivilizationCards>,
    )>,
    mut commands: Commands,
    mut recalculate_player_moves: MessageWriter<RecalculatePlayerMoves>,
    game_factions: Res<AvailableFactions>,
) {
    for build_city in command.read() {
        let has_architecture = player_query
            .get(build_city.player)
            .ok()
            .and_then(|(_, _, _, _, _, _, civ_cards)| civ_cards)
            .map(|c| c.owns(&CivCardName::Architecture))
            .unwrap_or(false);

        // Architecture (rule 25.3): save 1 token to treasury before returning the rest.
        // We must also update PlayerAreas since the token bypasses the normal ReturnTokenToStock path.
        if has_architecture {
            let saved_token =
                city_population
                    .get_mut(build_city.area)
                    .ok()
                    .and_then(|(mut pop, _)| {
                        pop.remove_tokens_from_area(&build_city.player, 1)
                            .and_then(|mut t| t.drain().next())
                    });
            if let Some(token) = saved_token
                && let Ok((_, mut player_areas, _, _, _, mut treasury, _)) =
                    player_query.get_mut(build_city.player)
            {
                treasury.add_token_to_treasury(token);
                player_areas.remove_token(token);
            }
        }

        if let Ok((mut population, _)) = city_population.get_mut(build_city.area) {
            return_all_tokens_from_area_to_players(&mut population, &mut commands);
        }

        if let Ok((mut city_stock, _, mut player_cities, _, faction, _, _)) =
            player_query.get_mut(build_city.player)
            && let Ok((_, area_transform)) = city_population.get_mut(build_city.area)
        {
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

pub fn on_enter_city_construction(
    player_query: Query<(Entity, &Faction), With<Player>>,
    mut commands: Commands,
    loading_from_save: Option<Res<LoadingFromSave>>,
) {
    info!("[CITY_CONSTRUCTION] Entering city construction phase");
    info!(
        "[CITY_CONSTRUCTION] LoadingFromSave present: {}",
        loading_from_save.is_some()
    );

    if let Some(ref save_state) = loading_from_save {
        info!(
            "[CITY_CONSTRUCTION] Save state - saved_activity: {:?}",
            save_state.saved_activity
        );
        info!(
            "[CITY_CONSTRUCTION] Save state - completed_factions: {:?}",
            save_state.completed_factions
        );
    }

    let total_players = player_query.iter().count();
    info!("[CITY_CONSTRUCTION] Total players: {}", total_players);

    let mut skipped = 0;
    let mut marked_for_building = 0;
    for (player_entity, faction) in player_query.iter() {
        if let Some(ref save_state) = loading_from_save
            && save_state.completed_factions.contains(&faction.faction)
        {
            info!(
                "[CITY_CONSTRUCTION] Skipping {:?} - already completed in save",
                faction.faction
            );
            skipped += 1;
            continue;
        }
        info!(
            "[CITY_CONSTRUCTION] Marking {:?} for building",
            faction.faction
        );
        commands.entity(player_entity).insert(IsBuilding);
        marked_for_building += 1;
    }

    info!(
        "[CITY_CONSTRUCTION] Summary: {} marked for building, {} skipped",
        marked_for_building, skipped
    );

    commands.insert_resource(CityConstructionPhaseActive);
    info!("[CITY_CONSTRUCTION] Inserted CityConstructionPhaseActive resource");

    if loading_from_save.is_some() {
        info!("[CITY_CONSTRUCTION] Removing LoadingFromSave resource");
        commands.remove_resource::<LoadingFromSave>();
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
