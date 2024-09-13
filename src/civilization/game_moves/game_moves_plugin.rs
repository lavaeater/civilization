use bevy::prelude::{in_state, App, Component, Entity, Event, EventReader, IntoSystemConfigs, Plugin, Query, Update};
use crate::GameActivity;

struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RecalculateMoves>()
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(
                Update, (
                    recalculate_pop_exp_moves.run_if(in_state(GameActivity::PopulationExpansion)),
                ),
            )
        ;
    }
}

#[derive(Event, Debug, Default)]
pub struct RecalculateMoves;


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
pub struct Move {
}

pub fn recalculate_pop_exp_moves(
    mut recalc_reader: EventReader<RecalculateMoves>,
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    mut player_query: Query<&Move>,
) {
    for event in recalc_reader.read() {
    }

    for event in recalc_player_reader.read() {
    }
}
