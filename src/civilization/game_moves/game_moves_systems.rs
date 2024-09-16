use bevy::prelude::{Commands, EventReader, Query};
use bevy::utils::HashMap;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use crate::civilization::game_moves::game_moves_events::RecalculatePlayerMoves;
use crate::civilization::general::general_components::{PlayerAreas, Population, PlayerStock};

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &PlayerStock)>,
    area_population_query: Query<&Population>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = HashMap::default();
        //Finally, we arrive at what we WANT to do!
        /* 
        OK, we know which areas we have tokens in. Those are areas
        that 
        */
        let mut command_index = 0;
        if let Ok((player_areas, stock)) = player_move_query.get(event.player) {
            for area in player_areas.areas().iter() {
                if let Ok(pop) = area_population_query.get(*area) {
                    command_index += 1;
                    moves.insert(command_index, Move::PopulationExpansion(
                        *area,
                        pop.max_expansion_for_player(event.player).min(stock.tokens_in_stock()),
                    ));
                }
            }
        }
        commands.entity(event.player).insert(AvailableMoves::new(moves));
    }
}