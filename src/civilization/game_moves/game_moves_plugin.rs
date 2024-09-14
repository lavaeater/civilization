use bevy::prelude::{in_state, App, Component, Entity, Event, EventReader, IntoSystemConfigs, OnEnter, Plugin, Query, Update};
use crate::GameActivity;

struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(
                OnEnter(GameActivity::PopulationExpansion), (
                    calculate_moves,
                ),
            ).add_systems(
            Update, (
                recalculate_pop_exp_moves_for_player.run_if(in_state(GameActivity::PopulationExpansion)),
            ),
        )
        ;
    }
}

#[derive(Event, Debug)]
pub struct RecalculatePlayerMoves {
    pub player: Entity,
}

impl RecalculatePlayerMoves {
    pub fn new(player: Entity) -> Self {
        RecalculatePlayerMoves {
            player
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct Move {}

pub fn calculate_moves(
    
) {
    
}

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    mut player_query: Query<&Move>,
) {
    for event in recalc_player_reader.read() {}
}
