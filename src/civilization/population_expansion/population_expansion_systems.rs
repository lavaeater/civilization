use crate::civilization::general::general_components::PlayerStock;
use crate::civilization::general::general_components::{PlayerAreas, Population};
use crate::civilization::general::general_events::MoveTokensFromStockToAreaCommand;
use crate::civilization::population_expansion::population_expansion_components::{ExpandAutomatically, ExpandManually, NeedsExpansion};
use crate::civilization::population_expansion::population_expansion_events::CheckPlayerExpansionEligibility;
use crate::GameActivity;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With};

pub fn check_area_population_expansion_eligibility(
    mut expansion_check_event: EventReader<CheckPlayerExpansionEligibility>,
    stock_query: Query<&PlayerStock, With<NeedsExpansion>>,
    population_query: Query<&Population>,
    mut commands: Commands,
) {
    for event in expansion_check_event.read() {
        if let Ok(stock) = stock_query.get(event.player) {
            let required_tokens = population_query
                .iter()
                .filter(|pop| pop.has_player(event.player))
                .map(|pop| pop.max_expansion_for_player(event.player))
                .sum::<usize>();
            
            if required_tokens > 0 {
                //This player still needs expansion, the area, maybe not
                commands.entity(event.player).insert(NeedsExpansion::default());
                if required_tokens <= stock.tokens_in_stock() {
                    commands
                        .entity(event.player)
                        .insert(ExpandAutomatically::new(required_tokens));
                } else {
                    commands
                        .entity(event.player)
                        .insert(ExpandManually::new(required_tokens));
                }
            }
        }
    }
}

pub fn check_population_expansion_eligibility(
    player_query: Query<(Entity, &PlayerAreas)>,
    mut commands: Commands,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>,
) {
    for (player, player_areas) in player_query.iter() {
        for area in player_areas.areas() {
            commands.entity(area).insert(NeedsExpansion::default());
        }
        if player_areas.has_any_population() {
            commands.entity(player).insert(NeedsExpansion::default());
            checker.send(CheckPlayerExpansionEligibility::new(player));
        }
    }
}

pub fn expand_population(
    area_query: Query<(Entity, &Population), With<NeedsExpansion>>,
    to_expand: Query<(Entity, &ExpandAutomatically), With<NeedsExpansion>>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut commands: Commands,
) {
    for (pop_entity, pop) in area_query.iter() {
        for (player, tokens) in pop.player_tokens.iter() {
            if to_expand.contains(*player) {
                match tokens.len() {
                    0 => {}
                    1 => {
                        event_writer.send(MoveTokensFromStockToAreaCommand {
                            area_entity: pop_entity,
                            player_entity: *player,
                            number_of_tokens: 1,
                        });
                    }
                    _ => {
                        event_writer.send(MoveTokensFromStockToAreaCommand {
                            area_entity: pop_entity,
                            player_entity: *player,
                            number_of_tokens: 2,
                        });
                    }
                }
            }
        }
        commands.entity(pop_entity).remove::<NeedsExpansion>();
    }
    for (player, _) in to_expand.iter() {
        commands.entity(player).remove::<NeedsExpansion>();
        commands.entity(player).remove::<ExpandAutomatically>();
    }
}

pub fn population_expansion_gate(
    gate_query: Query<&NeedsExpansion>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if gate_query.is_empty() {
        next_state.set(GameActivity::Census);
    }
}