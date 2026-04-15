use crate::civilization::components::{GameArea, PlayerAreas, SeaPassage, TokenStock, Treasury};
use crate::civilization::concepts::ships::ship_components::{PlayerShips, Ship, ShipStock};
use crate::civilization::concepts::ships::ship_ui_components::{
    AwaitingShipPlacement, ShipConstructionState,
};
use crate::loading::TextureAssets;
use crate::player::Player;
use crate::stupid_ai::IsHuman;
use crate::GameActivity;
use bevy::prelude::{
    Commands, Entity, Has, Name, NextState, Query, Res, ResMut, Sprite, Transform, With, info,
};

/// Called on entering `ShipConstruction`.
///
/// Pass 1 — Maintenance (rule 22.3): each ship costs 1 token from treasury or
/// stock; unpaid ships return to stock. Applies to all players.
///
/// Pass 2 — Building: AI players auto-build. Human players are paused here:
/// `ShipConstructionState` is populated and `AwaitingShipPlacement` is inserted;
/// `advance_ship_construction` waits until the human confirms before transitioning.
pub fn enter_ship_construction(
    mut player_query: Query<
        (
            Entity,
            &Name,
            &mut ShipStock,
            &mut PlayerShips,
            &mut Treasury,
            &mut TokenStock,
            &PlayerAreas,
            Has<IsHuman>,
        ),
        With<Player>,
    >,
    sea_passage_query: Query<Has<SeaPassage>>,
    area_transform_query: Query<&Transform, With<GameArea>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut ship_state: ResMut<ShipConstructionState>,
    textures: Res<TextureAssets>,
) {
    let mut human_needs_input = false;

    // ── Pass 1: Maintenance ───────────────────────────────────────────────────
    for (_player_entity, name, mut ship_stock, mut player_ships, mut treasury, mut token_stock, _, _) in
        player_query.iter_mut()
    {
        let areas_with_ships: Vec<Entity> = player_ships.all_areas_with_ships();
        for area in areas_with_ships {
            let paid = if treasury.tokens_in_treasury() >= 1 {
                treasury.remove_token_from_treasury();
                true
            } else if token_stock.tokens_in_stock() >= 1 {
                token_stock.remove_token_from_stock();
                true
            } else {
                false
            };

            if !paid
                && let Some(ship_entity) = player_ships.remove_ship_from_area(area)
            {
                ship_stock.return_ship(ship_entity);
                info!(
                    "[SHIPS] {} cannot maintain ship at {:?} — returned to stock",
                    name, area
                );
            }
        }
    }

    // ── Pass 2: Building ──────────────────────────────────────────────────────
    // TODO: sort by census order; Military holders go last (rule 22.11).
    for (player_entity, name, mut ship_stock, mut player_ships, mut treasury, mut token_stock, player_areas, is_human) in
        player_query.iter_mut()
    {
        let ships_on_board = player_ships.total_ships_on_board();
        if ships_on_board >= ShipStock::MAX_SHIPS {
            continue;
        }

        let can_afford = treasury.tokens_in_treasury() >= 2 || token_stock.tokens_in_stock() >= 2;
        if !can_afford {
            continue;
        }

        let max_buildable = ShipStock::MAX_SHIPS - ships_on_board;

        if is_human {
            // Gather areas with player tokens (preferring coastal ones).
            let mut available_areas: Vec<Entity> = player_areas
                .areas()
                .into_iter()
                .filter(|&a| sea_passage_query.get(a).unwrap_or(false))
                .collect();
            if available_areas.is_empty() {
                available_areas = player_areas.areas().into_iter().collect();
            }

            if available_areas.is_empty() || ship_stock.is_empty() {
                continue;
            }

            ship_state.populate(player_entity, available_areas, max_buildable);
            commands.entity(player_entity).insert(AwaitingShipPlacement);
            human_needs_input = true;
            info!("[SHIPS] Human player {:?} entering ship construction UI", player_entity);
        } else {
            // AI: build one ship in the first available area.
            let candidate_area = player_areas.areas().into_iter().next();
            let Some(area) = candidate_area else { continue };
            let Some(ship_entity) = ship_stock.take_ship() else { continue };

            if treasury.tokens_in_treasury() >= 2 {
                treasury.remove_token_from_treasury();
                treasury.remove_token_from_treasury();
            } else {
                token_stock.remove_token_from_stock();
                token_stock.remove_token_from_stock();
            }

            let area_pos = area_transform_query
                .get(area)
                .map(|t| t.translation.truncate())
                .unwrap_or_default();
            commands.entity(ship_entity).insert((
                Sprite {
                    image: textures.ship.clone(),
                    ..Default::default()
                },
                Transform::from_xyz(area_pos.x, area_pos.y, 2.0),
            ));

            player_ships.place_ship(area, ship_entity);
            info!(
                "[SHIPS] {} builds a ship at {:?} (fleet: {}/{})",
                name, area, player_ships.total_ships_on_board(), ShipStock::MAX_SHIPS
            );
        }
    }

    if !human_needs_input {
        next_state.set(GameActivity::Movement);
    }
}

/// Runs every frame during `ShipConstruction`.
/// Once no human player has `AwaitingShipPlacement`, applies the human's build
/// choices (from `ShipConstructionState`) and transitions to `Movement`.
pub fn advance_ship_construction(
    waiting: Query<Entity, With<AwaitingShipPlacement>>,
    mut player_query: Query<
        (&Name, &mut ShipStock, &mut PlayerShips, &mut Treasury, &mut TokenStock),
        With<Player>,
    >,
    area_transform_query: Query<&Transform, With<GameArea>>,
    mut ship_state: ResMut<ShipConstructionState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    textures: Res<TextureAssets>,
) {
    // Still waiting.
    if !waiting.is_empty() {
        return;
    }

    // Human confirmed (or there was no human) — apply the build plan.
    if let Some(player_entity) = ship_state.player {
        if let Ok((name, mut ship_stock, mut player_ships, mut treasury, mut token_stock)) =
            player_query.get_mut(player_entity)
        {
            let areas = ship_state.take_result(); // drains + clears resource
            for area in areas {
                let Some(ship_entity) = ship_stock.take_ship() else { break };

                if treasury.tokens_in_treasury() >= 2 {
                    treasury.remove_token_from_treasury();
                    treasury.remove_token_from_treasury();
                } else if token_stock.tokens_in_stock() >= 2 {
                    token_stock.remove_token_from_stock();
                    token_stock.remove_token_from_stock();
                } else {
                    // Can no longer afford — return ship and stop.
                    ship_stock.return_ship(ship_entity);
                    break;
                }

                let area_pos = area_transform_query
                    .get(area)
                    .map(|t| t.translation.truncate())
                    .unwrap_or_default();
                commands.entity(ship_entity).insert((
                    Sprite {
                        image: textures.ship.clone(),
                        ..Default::default()
                    },
                    Transform::from_xyz(area_pos.x, area_pos.y, 2.0),
                ));

                player_ships.place_ship(area, ship_entity);
                info!(
                    "[SHIPS] {} builds ship at {:?} (fleet: {}/{})",
                    name, area, player_ships.total_ships_on_board(), ShipStock::MAX_SHIPS
                );
            }
        }
    } else {
        ship_state.clear();
    }

    next_state.set(GameActivity::Movement);
}

/// Spawns the initial ship entities (stock) for a player during game setup.
pub fn create_ship_stock(commands: &mut Commands, player: Entity) -> (ShipStock, PlayerShips) {
    let ship_entities: Vec<Entity> = (0..ShipStock::MAX_SHIPS)
        .map(|_| {
            commands
                .spawn((
                    Name::new("Ship"),
                    Ship::new(player),
                ))
                .id()
        })
        .collect();

    (ShipStock::new(ship_entities), PlayerShips::default())
}
