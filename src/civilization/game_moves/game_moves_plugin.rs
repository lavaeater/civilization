use crate::civilization::general::general_components::{PlayerAreas, Population, Stock};
use crate::civilization::population_expansion::population_expansion_components::ExpandManually;
use crate::GameActivity;
use bevy::prelude::{in_state, App, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, OnAdd, Plugin, Query, Trigger, Update};

pub struct GameMovesPlugin;

impl Plugin for GameMovesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RecalculatePlayerMoves>()
            .add_systems(
                Update, (
                    recalculate_pop_exp_moves_for_player.run_if(in_state(GameActivity::PopulationExpansion)),
                ),
            )
            .observe(on_add_manual_expansion)
        ;
    }
}

pub fn on_add_manual_expansion(
    trigger: Trigger<OnAdd, ExpandManually>,
    mut event_writer: EventWriter<RecalculatePlayerMoves>,
) {
    event_writer.send(RecalculatePlayerMoves::new(trigger.entity()));
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
pub struct AvailableMoves {
    pub moves: Vec<Move>,
}

impl AvailableMoves {
    pub fn new(moves: Vec<Move>) -> Self {
        AvailableMoves {
            moves
        }
    }
}

#[derive(Debug)]
pub enum Move {
    PopulationExpansion(PopulationExpansionMove),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PopulationExpansionMove {
    pub command_index: i32,
    pub area: Entity,
    pub max_tokens: usize,
}

pub fn recalculate_pop_exp_moves_for_player(
    mut recalc_player_reader: EventReader<RecalculatePlayerMoves>,
    player_move_query: Query<(&PlayerAreas, &Stock)>,
    area_population_query: Query<&Population>,
    mut commands: Commands,
) {
    for event in recalc_player_reader.read() {
        commands.entity(event.player).remove::<AvailableMoves>();
        let mut moves = vec![];
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
                    moves.push(Move::PopulationExpansion(PopulationExpansionMove {
                        command_index,
                        area: *area,
                        max_tokens: pop.max_expansion_for_player(event.player).min(stock.tokens_in_stock()),
                    }));
                }
            }
        }
        commands.entity(event.player).insert(AvailableMoves::new(moves));
    }
}
