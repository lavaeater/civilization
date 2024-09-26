use crate::civilization::general::general_components::PlayerStock;
use crate::civilization::general::general_components::{PlayerAreas, Population};
use crate::civilization::general::general_events::MoveTokensFromStockToAreaCommand;
use crate::civilization::population_expansion::population_expansion_components::{AreaIsExpanding, ExpandAutomatically, ExpandManually, NeedsExpansion};
use crate::civilization::population_expansion::population_expansion_events::{CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand};
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};

pub fn check_area_population_expansion_eligibility(
    mut expansion_check_event: EventReader<CheckPlayerExpansionEligibility>,
    stock_query: Query<(&PlayerStock, &PlayerAreas, &NeedsExpansion)>,
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
                        .insert(ExpandAutomatically::default());
                } else {
                    commands
                        .entity(event.player)
                        .insert(ExpandManually::default());
                }
            }
        }
    }
}

pub fn enter_population_expansion(
    player_query: Query<(Entity, &PlayerAreas)>,
    area: Query<(Entity, &Population)>,
    mut commands: Commands,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
) {
    for (area_entity, pop) in area.iter() {
        commands.entity(area_entity).insert(AreaIsExpanding::new(pop.players()));
    }

    for (player, player_areas) in player_query.iter() {
        commands.entity(player).insert(NeedsExpansion::new(player_areas.areas()));
        checker.send(CheckPlayerExpansionEligibility::new(player));
    }
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
                if let Ok(mut expansion) = area_query.get_mut(*area) {
                    expansion.remove(player_entity);
                    if expansion.expansion_is_done() {
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
    player_gate_query: Query<&NeedsExpansion>,
    area_gate_query: Query<Entity, With<AreaIsExpanding>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // No players need expansion no more, so remove the NeedsExpansion component from all areas
    if player_gate_query.is_empty() {
        for area in area_gate_query.iter() {
            commands.entity(area).remove::<AreaIsExpanding>();
        }
        next_state.set(GameActivity::Census);
    }
}

// We hang here, bro!

pub fn expand_population_manually(
    mut area_query: Query<&mut AreaIsExpanding>,
    mut player_query: Query<&mut NeedsExpansion, With<ExpandManually>>,
    mut event_reader: EventReader<ExpandPopulationManuallyCommand>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
    mut commands: Commands,
) {
    for event in event_reader.read() {
        event_writer.send(MoveTokensFromStockToAreaCommand::new(event.area, event.player, event.number_of_tokens));
        if let Ok(mut expansion) = area_query.get_mut(event.area) {
            expansion.remove(event.player);
            if expansion.expansion_is_done() {
                commands.entity(event.area).remove::<AreaIsExpanding>();
            }
        }
        if let Ok(mut needs_expansion) = player_query.get_mut(event.player) {
            needs_expansion.remove(event.area);
        }
        checker.send(CheckPlayerExpansionEligibility::new(event.player));
    }
}