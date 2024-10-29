use crate::civilization::components::general_components::population::Population;
use crate::civilization::components::general_components::*;
use crate::civilization::events::general_events::MoveTokensFromStockToAreaCommand;
use crate::GameActivity;
use bevy::prelude::{debug, Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};
use crate::civilization::concepts::census::prelude::GameInfoAndStuff;
use crate::civilization::concepts::population_expansion::components::{AreaIsExpanding, ExpandAutomatically, ExpandManually, NeedsExpansion};
use crate::civilization::concepts::population_expansion::events::{CheckGate, CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand};

pub fn check_area_population_expansion_eligibility(
    mut expansion_check_event: EventReader<CheckPlayerExpansionEligibility>,
    stock_query: Query<(&TokenStock, &PlayerAreas, &NeedsExpansion)>,
    mut commands: Commands,
) {
    for event in expansion_check_event.read() {
        if let Ok((stock, player_areas, needs_expansion)) = stock_query.get(event.player) {
            if needs_expansion.areas_that_need_expansion.is_empty() || stock.is_empty() {
                commands.entity(event.player).remove::<NeedsExpansion>();
                commands.entity(event.player).remove::<ExpandManually>();
                commands.entity(event.player).remove::<ExpandAutomatically>();
            } else if player_areas.required_tokens_for_expansion() > 0 {
                if player_areas.required_tokens_for_expansion() <= stock.tokens_in_stock() {
                    commands
                        .entity(event.player)
                        .insert(ExpandAutomatically);
                } else {
                    commands
                        .entity(event.player)
                        .insert(ExpandManually);
                }
            }
        }
    }
}

pub fn enter_population_expansion(
    player_query: Query<(Entity, &PlayerAreas)>,
    area: Query<(Entity, &Population)>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
) {
    game_info.round += 1;
    debug!("Entering population expansion round {}", game_info.round);
    for (area_entity, pop) in area.iter() {
        debug!("Add expansion component to an area");
        if pop.has_population() {
            commands.entity(area_entity).insert(AreaIsExpanding::new(pop.players()));
        }
    }

    for (player, player_areas) in player_query.iter() {
        debug!("Add expansion component to a player");
        commands.entity(player).insert(NeedsExpansion::new(player_areas.areas()));
        checker.send(CheckPlayerExpansionEligibility::new(player));
    }
    debug!("We are DONE! with the startup! {}", game_info.round);
}

pub fn auto_expand_population(
    mut area_query: Query<&mut AreaIsExpanding>,
    mut player_query: Query<(Entity, &PlayerAreas, &mut NeedsExpansion), With<ExpandAutomatically>>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut commands: Commands,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
) {
    for (player_entity, player_areas, mut needs_expansion) in player_query.iter_mut() {
        for area in needs_expansion.areas_that_need_expansion.iter() {
            let needed_tokens = player_areas.required_tokens_for_expansion_for_area(*area);
            if needed_tokens > 0 {
                event_writer.send(MoveTokensFromStockToAreaCommand::new(*area, player_entity, needed_tokens));
                if let Ok(mut area_expansion) = area_query.get_mut(*area) {
                    area_expansion.remove(player_entity);
                    if area_expansion.expansion_is_done() {
                        commands.entity(*area).remove::<AreaIsExpanding>();
                    }
                }
            }
        }
        needs_expansion.areas_that_need_expansion.clear();
        checker.send(CheckPlayerExpansionEligibility::new(player_entity));
    }
}

pub fn population_expansion_gate(
    mut check_gate: EventReader<CheckGate>,
    player_gate_query: Query<Entity, With<NeedsExpansion>>,
    area_gate_query: Query<Entity, With<AreaIsExpanding>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>
) {
    for _ in check_gate.read() {
        debug!("Checking gate");
        // No players need expansion no more, so remove the NeedsExpansion component from all areas
        if player_gate_query.is_empty() {
            debug!("No players need expansion, let's do census!");
            for area in area_gate_query.iter() {
                commands.entity(area).remove::<AreaIsExpanding>();
            }
            next_state.set(GameActivity::Census);
        } else {
            debug!("Players still need expansion");
        }
    }
}

pub fn expand_population_manually(
    mut area_query: Query<&mut AreaIsExpanding>,
    mut player_query: Query<&mut NeedsExpansion, With<ExpandManually>>,
    mut event_reader: EventReader<ExpandPopulationManuallyCommand>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
    mut commands: Commands,
) {
    for event in event_reader.read() {
        // debug!("Expanding population manually: {:?}", event);
        // debug!("Player components:");
        // commands.entity(event.player).log_components();
        // debug!("Area components:");
        // commands.entity(event.area).log_components();
        event_writer.send(MoveTokensFromStockToAreaCommand::new(event.area, event.player, event.number_of_tokens));
        
        /*
        The missing piece of the puzzle is that we must remove the expandmanually component from the player
        to make sure we generate the available moves for the player again!
         */
        commands.entity(event.player).remove::<ExpandManually>();
        if let Ok(mut area_expansion) = area_query.get_mut(event.area) {
            // debug!("Removing player from expansion list");
            area_expansion.remove(event.player);
            if area_expansion.expansion_is_done() {
                // debug!("Area expansion is done but we don't remove the component");
                // commands.entity(event.area).remove::<AreaIsExpanding>();
            }
        }
        if let Ok(mut needs_expansion) = player_query.get_mut(event.player) {
            // debug!("Remove area from player's expansion list");
            needs_expansion.remove(event.area);
        }
        // debug!("Checking player expansion eligibility");
        checker.send(CheckPlayerExpansionEligibility::new(event.player));
    }
}