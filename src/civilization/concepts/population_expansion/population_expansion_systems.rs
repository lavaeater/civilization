use crate::civilization::components::*;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::population_expansion::population_expansion_components::{
    AreaIsExpanding, ExpandAutomatically, ExpandManually, NeedsExpansion,
    PopExpAreaHighlight, PopExpHighlightMarker,
};
use crate::civilization::concepts::population_expansion::population_expansion_events::{
    CheckGate, CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand,
};
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::loading::TextureAssets;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::prelude::{
    debug, default, info, ButtonInput, Camera, Commands, Entity, GlobalTransform, Has,
    MessageReader, MessageWriter, MouseButton, NextState, Query, Res, ResMut, Sprite, Transform,
    Vec3, Window, With, Without,
};
use bevy::window::PrimaryWindow;

pub fn check_area_population_expansion_eligibility(
    mut expansion_check_event: MessageReader<CheckPlayerExpansionEligibility>,
    stock_query: Query<(&TokenStock, &PlayerAreas, &NeedsExpansion)>,
    mut commands: Commands,
) {
    for event in expansion_check_event.read() {
        if let Ok((stock, player_areas, needs_expansion)) = stock_query.get(event.player) {
            if needs_expansion.areas_that_need_expansion.is_empty() || stock.is_empty() {
                commands.entity(event.player).remove::<NeedsExpansion>();
                commands.entity(event.player).remove::<ExpandManually>();
                commands
                    .entity(event.player)
                    .remove::<ExpandAutomatically>();
            } else if player_areas.required_tokens_for_expansion() > 0 {
                if player_areas.required_tokens_for_expansion() <= stock.tokens_in_stock() {
                    commands.entity(event.player).insert(ExpandAutomatically);
                } else {
                    commands.entity(event.player).insert(ExpandManually);
                }
            }
        }
    }
}

pub fn enter_population_expansion(
    player_query: Query<(Entity, &PlayerAreas, Has<IsHuman>)>,
    area: Query<(Entity, &Population)>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut checker: MessageWriter<CheckPlayerExpansionEligibility>,
) {
    game_info.round += 1;
    info!("[POP_EXP] Entering population expansion phase, round {}", game_info.round);
    
    let mut areas_with_pop = 0;
    for (area_entity, pop) in area.iter() {
        if pop.has_population() {
            areas_with_pop += 1;
            commands
                .entity(area_entity)
                .insert(AreaIsExpanding::new(pop.players()));
        }
    }
    info!("[POP_EXP] {} areas have population and need expansion", areas_with_pop);

    let mut human_count = 0;
    let mut ai_count = 0;
    for (player, player_areas, is_human) in player_query.iter() {
        if is_human {
            human_count += 1;
        } else {
            ai_count += 1;
        }
        commands
            .entity(player)
            .insert(NeedsExpansion::new(player_areas.areas()));
        checker.write(CheckPlayerExpansionEligibility::new(player));
    }
    info!("[POP_EXP] {} human players, {} AI players need expansion check", human_count, ai_count);
}

pub fn auto_expand_population(
    mut area_query: Query<&mut AreaIsExpanding>,
    mut player_query: Query<(Entity, &PlayerAreas, &mut NeedsExpansion), With<ExpandAutomatically>>,
    mut event_writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
    mut commands: Commands,
    mut checker: MessageWriter<CheckPlayerExpansionEligibility>,
) {
    for (player_entity, player_areas, mut needs_expansion) in player_query.iter_mut() {
        for area in needs_expansion.areas_that_need_expansion.iter() {
            let needed_tokens = player_areas.required_tokens_for_expansion_for_area(*area);
            if needed_tokens > 0 {
                event_writer.write(MoveTokensFromStockToAreaCommand::new(
                    *area,
                    player_entity,
                    needed_tokens,
                ));
                if let Ok(mut area_expansion) = area_query.get_mut(*area) {
                    area_expansion.remove(player_entity);
                    if area_expansion.expansion_is_done() {
                        commands.entity(*area).remove::<AreaIsExpanding>();
                    }
                }
            }
        }
        needs_expansion.areas_that_need_expansion.clear();
        checker.write(CheckPlayerExpansionEligibility::new(player_entity));
    }
}

pub fn population_expansion_gate(
    mut check_gate: MessageReader<CheckGate>,
    player_gate_query: Query<(Entity, Has<IsHuman>, Has<ExpandManually>, Has<ExpandAutomatically>), With<NeedsExpansion>>,
    area_gate_query: Query<Entity, With<AreaIsExpanding>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    for _ in check_gate.read() {
        let players_needing_expansion: Vec<_> = player_gate_query.iter().collect();
        
        if players_needing_expansion.is_empty() {
            info!("[POP_EXP] All players done with expansion, transitioning to Census");
            for area in area_gate_query.iter() {
                commands.entity(area).remove::<AreaIsExpanding>();
            }
            next_state.set(GameActivity::Census);
        } else {
            // Log who is still waiting
            for (entity, is_human, has_manual, has_auto) in players_needing_expansion.iter() {
                info!(
                    "[POP_EXP] Player {:?} still needs expansion: human={}, manual={}, auto={}",
                    entity, is_human, has_manual, has_auto
                );
            }
        }
    }
}

pub fn expand_population_manually(
    mut area_query: Query<&mut AreaIsExpanding>,
    mut player_query: Query<&mut NeedsExpansion, With<ExpandManually>>,
    mut event_reader: MessageReader<ExpandPopulationManuallyCommand>,
    mut event_writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
    mut checker: MessageWriter<CheckPlayerExpansionEligibility>,
    mut commands: Commands,
) {
    for event in event_reader.read() {
        event_writer.write(MoveTokensFromStockToAreaCommand::new(
            event.area,
            event.player,
            event.number_of_tokens,
        ));

        /*
        The missing piece of the puzzle is that we must remove the expandmanually component from the player
        to make sure we generate the available moves for the player again!
         */
        commands.entity(event.player).remove::<ExpandManually>();
        if let Ok(mut area_expansion) = area_query.get_mut(event.area) {
            area_expansion.remove(event.player);
            if area_expansion.expansion_is_done() {
                // //debug!("Area expansion is done but we don't remove the component");
                // commands.entity(event.area).remove::<AreaIsExpanding>();
            }
        }
        if let Ok(mut needs_expansion) = player_query.get_mut(event.player) {
            needs_expansion.remove(event.area);
        }
        checker.write(CheckPlayerExpansionEligibility::new(event.player));
    }
}

/// When a human player gets AvailableMoves with PopExp moves, mark those areas with highlights.
pub fn highlight_pop_exp_areas_for_human(
    human_players: Query<(Entity, &AvailableMoves), With<IsHuman>>,
    area_query: Query<(Entity, &Transform), (With<GameArea>, Without<PopExpAreaHighlight>)>,
    mut commands: Commands,
    textures: Res<TextureAssets>,
) {
    for (player_entity, available_moves) in human_players.iter() {
        for (_index, game_move) in available_moves.moves.iter() {
            if let GameMove::PopulationExpansion(pop_exp_move) = game_move {
                // Mark the area with highlight component if not already marked
                if let Ok((area_entity, area_transform)) = area_query.get(pop_exp_move.area) {
                    debug!("Highlighting area {:?} for PopExp", area_entity);
                    
                    // Add highlight component to the area
                    commands.entity(area_entity).insert(
                        PopExpAreaHighlight::new(player_entity, pop_exp_move.max_tokens),
                    );
                    
                    // Spawn a visual marker sprite at the area's position
                    commands.spawn((
                        PopExpHighlightMarker { area: area_entity },
                        Sprite {
                            image: textures.dot.clone(),
                            color: bevy::prelude::Color::srgba(0.0, 1.0, 0.0, 0.7),
                            ..default()
                        },
                        Transform::from_translation(
                            area_transform.translation + Vec3::new(0.0, 0.0, 0.5),
                        )
                        .with_scale(Vec3::splat(0.5)),
                    ));
                }
            }
        }
    }
}

/// Remove highlight markers when the human player no longer has AvailableMoves.
pub fn cleanup_pop_exp_highlights(
    human_players: Query<Entity, (With<IsHuman>, Without<AvailableMoves>)>,
    highlighted_areas: Query<(Entity, &PopExpAreaHighlight)>,
    highlight_markers: Query<(Entity, &PopExpHighlightMarker)>,
    mut commands: Commands,
) {
    for player_entity in human_players.iter() {
        // Remove highlight components from areas belonging to this player
        for (area_entity, highlight) in highlighted_areas.iter() {
            if highlight.player == player_entity {
                commands.entity(area_entity).remove::<PopExpAreaHighlight>();
            }
        }
    }
    
    // Despawn orphaned markers (areas no longer highlighted)
    for (marker_entity, marker) in highlight_markers.iter() {
        if highlighted_areas.get(marker.area).is_err() {
            commands.entity(marker_entity).despawn();
        }
    }
}

/// Handle mouse clicks on highlighted PopExp areas to execute the expansion.
pub fn handle_pop_exp_area_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    highlighted_areas: Query<(Entity, &Transform, &PopExpAreaHighlight), With<GameArea>>,
    mut expand_writer: MessageWriter<ExpandPopulationManuallyCommand>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };
    
    // Check if click is near any highlighted area (within a radius)
    const CLICK_RADIUS: f32 = 30.0;
    
    for (area_entity, area_transform, highlight) in highlighted_areas.iter() {
        let area_pos = area_transform.translation.truncate();
        let distance = world_pos.distance(area_pos);
        
        if distance <= CLICK_RADIUS {
            debug!(
                "Clicked on highlighted area {:?}, expanding with {} tokens",
                area_entity, highlight.max_tokens
            );
            
            expand_writer.write(ExpandPopulationManuallyCommand::new(
                highlight.player,
                area_entity,
                highlight.max_tokens,
            ));
            return;
        }
    }
}
