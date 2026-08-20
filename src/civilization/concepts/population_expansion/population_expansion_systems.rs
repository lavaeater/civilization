use crate::GameActivity;
use crate::civilization::components::*;
use crate::civilization::concepts::census::GameInfoAndStuff;
use crate::civilization::concepts::population_expansion::population_expansion_components::{
    AreaIsExpanding, ExpandAutomatically, ExpandManually, NeedsExpansion, PopExpAreaHighlight,
    PopExpHighlightMarker,
};
use crate::civilization::concepts::population_expansion::population_expansion_events::{
    CheckGate, CheckPlayerExpansionEligibility, ExpandPopulationManuallyCommand,
};
use crate::civilization::concepts::save_game::LoadingFromSave;
use crate::civilization::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::game_moves::{AvailableMoves, GameMove};
use crate::loading::TextureAssets;
use crate::stupid_ai::IsHuman;
use bevy::prelude::{
    ButtonInput, Camera, Commands, Entity, GlobalTransform, Has, MessageReader, MessageWriter,
    MouseButton, Name, NextState, Query, Res, ResMut, Sprite, Transform, Vec3, Window, With,
    Without, debug, default, info,
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

/// Per-round diagnostic dump. Runs at the top of every round so late-game stalls
/// (e.g. players stuck on the A.S.T. with no cities) can be reasoned about from the
/// log alone: city count, supportable cities, total population, stock, treasury and
/// A.S.T. position for each player.
pub fn log_round_state(
    game_info: Res<GameInfoAndStuff>,
    player_query: Query<(
        Entity,
        &Name,
        &PlayerCities,
        &PlayerAreas,
        &TokenStock,
        &Treasury,
        Option<&crate::civilization::concepts::succession::AstPosition>,
    )>,
    token_query: Query<&Token>,
    area_pop_query: Query<(Entity, &Population, Option<&Name>), With<GameArea>>,
) {
    // Ground-truth token count per owner, straight from the Token entities. This is
    // the conservation check: stock + on-board + treasury *should* equal this, and
    // it should stay constant (47/player) all game. A drop means tokens are leaking;
    // a mismatch with PlayerAreas means PlayerAreas has drifted out of sync.
    let mut owned_tokens: bevy::platform::collections::HashMap<Entity, usize> =
        bevy::platform::collections::HashMap::default();
    for token in token_query.iter() {
        *owned_tokens.entry(token.player()).or_insert(0) += 1;
    }

    info!("[STATE] ───── Round {} ─────", game_info.round);
    for (entity, name, cities, areas, stock, treasury, ast) in player_query.iter() {
        let n_cities = cities.number_of_cities();
        let total_pop = areas.total_population();
        let supportable = total_pop / 2; // each city needs 2 population (rule)
        let (space, epoch) = ast.map_or((-1, "—"), |p| (i64::from(p.space), p.epoch().name()));
        let real_tokens = owned_tokens.get(&entity).copied().unwrap_or(0);
        let stock_n = stock.tokens_in_stock();
        let treasury_n = treasury.tokens_in_treasury();
        // accounted = what we can see (board + reserve + paid taxes); should == real_tokens.
        let accounted = total_pop + stock_n + treasury_n;

        // Cross-check the board: sum this player's tokens straight from each area's
        // Population, and flag areas where Population and PlayerAreas disagree. This
        // localises the desync to a specific area (we suspect city areas).
        let mut board_pop = 0usize;
        let mut mismatches: Vec<String> = Vec::new();
        for (area_entity, population, area_name) in area_pop_query.iter() {
            let in_pop = population.population_for_player(entity);
            let in_pa = areas.population_in_area(area_entity);
            board_pop += in_pop;
            if in_pop != in_pa {
                let label = area_name.map_or_else(
                    || format!("{area_entity:?}"),
                    std::string::ToString::to_string,
                );
                mismatches.push(format!("{label}(pop={in_pop},pa={in_pa})"));
            }
        }

        info!(
            "[STATE] {name}: AST sp{space} ({epoch}) | cities={n_cities} (supportable={supportable}) \
             | pa_pop={total_pop} board_pop={board_pop} areas={areas_n} | stock={stock_n} treasury={treasury_n} \
             | tokens={real_tokens} accounted={accounted}{flag}",
            areas_n = areas.areas().len(),
            flag = if accounted == real_tokens {
                ""
            } else {
                " ⚠DESYNC"
            },
        );
        if !mismatches.is_empty() {
            info!(
                "[STATE]   ↳ area desync for {name}: {}",
                mismatches.join(", ")
            );
        }
    }
}

pub fn enter_population_expansion(
    player_query: Query<(Entity, &Faction, &PlayerAreas, Has<IsHuman>)>,
    area: Query<(Entity, &Population)>,
    mut game_info: ResMut<GameInfoAndStuff>,
    mut commands: Commands,
    mut checker: MessageWriter<CheckPlayerExpansionEligibility>,
    loading_from_save: Option<Res<LoadingFromSave>>,
) {
    game_info.round += 1;
    info!(
        "[POP_EXP] Entering population expansion phase, round {}",
        game_info.round
    );

    let mut areas_with_pop = 0;
    for (area_entity, pop) in area.iter() {
        if pop.has_population() {
            areas_with_pop += 1;
            commands
                .entity(area_entity)
                .insert(AreaIsExpanding::new(pop.players()));
        }
    }
    info!(
        "[POP_EXP] {} areas have population and need expansion",
        areas_with_pop
    );

    let mut human_count = 0;
    let mut ai_count = 0;
    let mut skipped_count = 0;
    for (player, faction, player_areas, is_human) in player_query.iter() {
        // Skip players that already completed expansion in the saved game
        if let Some(ref save_state) = loading_from_save
            && save_state.completed_factions.contains(&faction.faction)
        {
            info!(
                "[POP_EXP] Skipping {:?} - already completed expansion in save",
                faction.faction
            );
            skipped_count += 1;
            continue;
        }
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
    info!(
        "[POP_EXP] {} human, {} AI need expansion, {} skipped (already done)",
        human_count, ai_count, skipped_count
    );

    // Clean up LoadingFromSave now that we've used it
    if loading_from_save.is_some() {
        commands.remove_resource::<LoadingFromSave>();
    }
}

pub fn auto_expand_population(
    mut area_query: Query<&mut AreaIsExpanding>,
    mut player_query: Query<(Entity, &PlayerAreas, &mut NeedsExpansion), With<ExpandAutomatically>>,
    mut event_writer: MessageWriter<MoveTokensFromStockToAreaCommand>,
    mut commands: Commands,
    mut checker: MessageWriter<CheckPlayerExpansionEligibility>,
) {
    for (player_entity, player_areas, mut needs_expansion) in &mut player_query {
        for area in &needs_expansion.areas_that_need_expansion {
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
    player_gate_query: Query<
        (
            Entity,
            Has<IsHuman>,
            Has<ExpandManually>,
            Has<ExpandAutomatically>,
        ),
        With<NeedsExpansion>,
    >,
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
            for (entity, is_human, has_manual, has_auto) in &players_needing_expansion {
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

/// When a human player's PopExp `AvailableMoves` (re)computes, sync highlights
/// to exactly match it: add markers for newly-offered areas, refresh the
/// `max_tokens` on ones already highlighted, and drop highlights for areas no
/// longer offered (i.e. already expanded this round via
/// `recalculate_pop_exp_moves_for_player`'s `NeedsExpansion` filter) so a
/// player can't click a stale marker and expand the same area twice.
pub fn highlight_pop_exp_areas_for_human(
    human_players: Query<
        (Entity, &AvailableMoves),
        (With<IsHuman>, bevy::prelude::Changed<AvailableMoves>),
    >,
    area_query: Query<(Entity, &Transform), With<GameArea>>,
    highlighted_areas: Query<(Entity, &PopExpAreaHighlight)>,
    highlight_markers: Query<&PopExpHighlightMarker>,
    mut commands: Commands,
    textures: Res<TextureAssets>,
) {
    for (player_entity, available_moves) in human_players.iter() {
        let mut offered_areas: bevy::platform::collections::HashSet<Entity> = default();

        for (_index, game_move) in &available_moves.moves {
            if let GameMove::PopulationExpansion(pop_exp_move) = game_move {
                offered_areas.insert(pop_exp_move.area);
                let Ok((area_entity, area_transform)) = area_query.get(pop_exp_move.area) else {
                    continue;
                };

                // Refresh (or add) the highlight; cheap even when unchanged.
                commands
                    .entity(area_entity)
                    .insert(PopExpAreaHighlight::new(
                        player_entity,
                        pop_exp_move.max_tokens,
                    ));

                // Only spawn a marker sprite the first time this area is highlighted.
                let already_marked = highlight_markers.iter().any(|m| m.area == area_entity);
                if !already_marked {
                    debug!("Highlighting area {:?} for PopExp", area_entity);
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

        // Drop highlights for areas this player no longer has a move for
        // (already expanded this round) so they can't be clicked again.
        for (area_entity, highlight) in &highlighted_areas {
            if highlight.player == player_entity && !offered_areas.contains(&area_entity) {
                commands.entity(area_entity).remove::<PopExpAreaHighlight>();
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
    available_moves_query: Query<&AvailableMoves>,
    mut expand_writer: MessageWriter<ExpandPopulationManuallyCommand>,
) {
    // Click is considered a hit if it lands within this radius of an area.
    const CLICK_RADIUS: f32 = 30.0;

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    for (area_entity, area_transform, highlight) in highlighted_areas.iter() {
        let area_pos = area_transform.translation.truncate();
        let distance = world_pos.distance(area_pos);

        if distance <= CLICK_RADIUS {
            // Defense in depth: only honor the click if this area is still
            // an actual pending move for this player right now (belt-and-
            // suspenders alongside the highlight-sync fix above -- a stale
            // highlight should never let an area be expanded twice).
            let still_valid = available_moves_query.get(highlight.player).is_ok_and(|moves| {
                moves.moves.values().any(|m| {
                    matches!(m, GameMove::PopulationExpansion(pop_exp_move) if pop_exp_move.area == area_entity)
                })
            });
            if !still_valid {
                debug!(
                    "Ignoring click on stale PopExp highlight for area {:?}",
                    area_entity
                );
                return;
            }

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
