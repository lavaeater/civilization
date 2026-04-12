use crate::civilization::components::{PlayerAreas, TokenStock, Treasury};
use crate::civilization::concepts::ships::ship_components::{PlayerShips, Ship, ShipStock};
use crate::player::Player;
use crate::GameActivity;
use bevy::prelude::{
    Commands, Entity, Name, NextState, Query, Res, ResMut, Sprite, Transform, With,
};
use crate::loading::TextureAssets;

/// Called on entering `ShipConstruction`.
///
/// Performs in one pass (both are instantaneous since there is no human UI yet):
/// 1. **Maintenance** (rule 22.3): each ship on the board costs 1 token from treasury
///    or stock. Unpaid ships return to stock.
/// 2. **Building** (rules 22.1–22.4): AI players build new ships up to the fleet cap.
///    Human player construction is not yet interactive — the human also auto-builds
///    for now.
///
/// Transitions to `Movement` when done.
pub fn enter_ship_construction(
    mut player_query: Query<
        (Entity, &Name, &mut ShipStock, &mut PlayerShips, &mut Treasury, &mut TokenStock, &PlayerAreas),
        With<Player>,
    >,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameActivity>>,
    textures: Res<TextureAssets>,
) {
    // ── 1. Maintenance ────────────────────────────────────────────────────────
    for (_player_entity, name, mut ship_stock, mut player_ships, mut treasury, mut token_stock, _) in
        player_query.iter_mut()
    {
        let areas_with_ships: Vec<Entity> = player_ships.all_areas_with_ships();
        for area in areas_with_ships {
            // Pay 1 token from treasury first, then stock; otherwise return ship.
            let paid = if treasury.tokens_in_treasury() >= 1 {
                treasury.remove_token_from_treasury();
                true
            } else if token_stock.tokens_in_stock() >= 1 {
                token_stock.remove_token_from_stock();
                true
            } else {
                false
            };

            if !paid {
                // Return ship to stock — player cannot maintain it.
                if let Some(ship_entity) = player_ships.remove_ship_from_area(area) {
                    ship_stock.return_ship(ship_entity);
                    bevy::prelude::info!(
                        "[SHIPS] {} cannot maintain ship at {:?} — returned to stock",
                        name, area
                    );
                }
            }
        }
    }

    // ── 2. Building ───────────────────────────────────────────────────────────
    // Build in census order (most tokens first). For now just iterate all players.
    // TODO: sort by census order; Military holders go last (rule 22.11).
    for (_player_entity, name, mut ship_stock, mut player_ships, mut treasury, mut token_stock, player_areas) in
        player_query.iter_mut()
    {
        let ships_on_board = player_ships.total_ships_on_board();
        let total_fleet = ships_on_board + ship_stock.count_in_stock();
        if total_fleet >= ShipStock::MAX_SHIPS {
            continue; // Already at cap.
        }

        // Simple AI heuristic: build if we can afford it and have a coastal area.
        let can_afford_treasury = treasury.tokens_in_treasury() >= 2;
        let can_afford_levy = token_stock.tokens_in_stock() >= 2;

        if !can_afford_treasury && !can_afford_levy {
            continue;
        }

        // Find an area where the player has units to place the ship.
        let candidate_area = player_areas.areas().into_iter().next();
        let Some(area) = candidate_area else { continue };

        // Take a ship from stock; if none, this player has no ships to deploy.
        let Some(ship_entity) = ship_stock.take_ship() else { continue };

        // Deduct the construction cost.
        if can_afford_treasury {
            treasury.remove_token_from_treasury();
            treasury.remove_token_from_treasury();
        } else {
            token_stock.remove_token_from_stock();
            token_stock.remove_token_from_stock();
        }

        // Place the ship entity on the board and add a sprite.
        commands.entity(ship_entity).insert((
            Sprite {
                image: textures.ship.clone(),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 2.0), // position updated by fix_token_positions
        ));

        player_ships.place_ship(area, ship_entity);

        bevy::prelude::info!(
            "[SHIPS] {} builds a ship at {:?} (fleet: {}/{})",
            name, area, player_ships.total_ships_on_board(), ShipStock::MAX_SHIPS
        );
    }

    next_state.set(GameActivity::Movement);
}

/// Spawns the initial ship entities (stock) for a player during game setup.
/// Call this after spawning the player entity.
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
