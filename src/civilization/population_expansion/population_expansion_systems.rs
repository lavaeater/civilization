use bevy::prelude::{Commands, Entity, EventReader, EventWriter, NextState, Query, ResMut, With, Without};
use bevy_inspector_egui::egui::Area;
use crate::civilization::general::general_components::{PlayerAreas, Population};
use crate::civilization::general::general_events::MoveTokensFromStockToAreaCommand;
use crate::civilization::general::general_components::PlayerStock;
use crate::civilization::population_expansion::population_expansion_components::{ExpandAutomatically, ExpandManually, NeedsExpansion};
use crate::civilization::population_expansion::population_expansion_events::CheckPlayerExpansionEligibility;
use crate::GameActivity;
use crate::player::Player;

pub fn check_area_population_expansion_eligibility(
    mut expansion_check_event: EventReader<CheckPlayerExpansionEligibility>,
    player_query: Query<(&PlayerAreas, &PlayerStock)>,
    area_query: Query<&Population>,
) {
    for event in expansion_check_event.iter() {
        if let Ok((player_areas, stock)) = player_query.get(event.player) {
            for area in player_areas.areas().iter() {
                if let Ok(pop) = area_query.get(*area) {
                    let mut required_tokens = 0;
                    for (pop_entity, pop) in area_query.iter() {
                        if let Some(p) = pop.player_tokens.get(&player) {
                            let rt = match p.len() {
                                1 => { 1 }
                                0 => { 0 }
                                _ => { 2 }
                            };
                            if rt > 0 {
                                commands.entity(pop_entity).insert(NeedsExpansion {});
                            }
                    
                            required_tokens += rt;
                        };
                    }
                    if required_tokens > 0 {
                        if required_tokens <= tokens_in_stock {
                            commands
                                .entity(player)
                                .insert((ExpandAutomatically::new(required_tokens), NeedsExpansion {}));
                        } else {
                            commands
                                .entity(player)
                                .insert((ExpandManually::new(required_tokens), NeedsExpansion {}));
                        }
                    }
                }
            }
        }
    }
    // // how many tokens has the player?
    
}

pub fn check_population_expansion_eligibility(
    player_query: Query<(Entity, &PlayerAreas)>,
    mut commands: Commands,
    mut checker: EventWriter<CheckPlayerExpansionEligibility>
) {
    for (player, areas) in player_query.iter() {
        if areas
        commands.entity(player).insert(NeedsExpansion::default());
        
        for area in area_query.iter() {
            commands.entity(area).insert(NeedsExpansion::default());
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