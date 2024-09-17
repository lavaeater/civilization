use crate::civilization::general::general_components::PlayerStock;
use crate::civilization::general::general_components::{PlayerAreas, Population};
use crate::civilization::general::general_events::MoveTokensFromStockToAreaCommand;
use crate::civilization::population_expansion::population_expansion_components::{AreaIsExpanding, ExpandAutomatically, ExpandManually, NeedsExpansion};
use crate::civilization::population_expansion::population_expansion_events::{CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand};
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};
use crate::player::Player;

pub fn check_area_population_expansion_eligibility(
    mut expansion_check_event: EventReader<CheckPlayerExpansionEligibility>,
    stock_query: Query<(&PlayerStock, &PlayerAreas), With<NeedsExpansion>>,
    mut commands: Commands,
) {
    for event in expansion_check_event.read() {
        if let Ok((stock, player_areas)) = stock_query.get(event.player) {
            if player_areas.required_tokens_for_expansion() > 0 {
                if player_areas.required_tokens_for_expansion() <= stock.tokens_in_stock() {
                    commands
                        .entity(event.player)
                        .insert(ExpandAutomatically::default());
                } else {
                    commands
                        .entity(event.player)
                        .insert(ExpandManually::new(player_areas.required_tokens_for_expansion()));
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
) {
    for (player_entity, player_areas, mut needs_expansion) in player_query.iter_mut() {
        for area in needs_expansion.areas_that_need_expansion.iter() {
            let needed_tokens = player_areas.required_tokens_for_expansion_for_area(*area);
            if needed_tokens > 0 {
                event_writer.send(MoveTokensFromStockToAreaCommand::new(*area, player_entity, needed_tokens));
                needs_expansion.remove(*area); //Will it work?
                if let Ok(mut expansion) = area_query.get_mut(*area) {
                    expansion.remove(player_entity);
                    if expansion.expansion_is_done() {
                        commands.entity(*area).remove::<AreaIsExpanding>();
                    }
                }
            }
        }
        if needs_expansion.areas_that_need_expansion.is_empty() {
            commands.entity(player_entity).remove::<ExpandAutomatically>();
        }
    }
}

pub fn population_expansion_gate(
    player_gate_query: Query<&NeedsExpansion>,
    area_gate_query: Query<&AreaIsExpanding>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    // No players need expansion no more, so remove the NeedsExpansion component from all areas
    if player_gate_query.is_empty() && area_gate_query.is_empty() {
        next_state.set(GameActivity::Census);
    }
}

pub fn expand_population_manually(
    mut event_reader: EventReader<ExpandPopulationManuallyCommand>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
) {
    for event in event_reader.read() {
        event_writer.send(MoveTokensFromStockToAreaCommand::new(event.area, event.player, event.number_of_tokens));
        checker.send(CheckPlayerExpansionEligibility::new(event.player));
    }
}