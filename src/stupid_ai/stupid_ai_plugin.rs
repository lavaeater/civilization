use bevy::app::{Plugin, Update};
use bevy::prelude::{in_state, App, Commands, Component, Entity, Event, EventReader, EventWriter, Has, IntoSystemConfigs, OnAdd, Query, Reflect, Trigger};
use crate::GameState;
use crate::civilization::game_moves::game_moves_components::{AvailableMoves, Move};
use crate::civilization::general::general_components::{BuiltCity, PlayerAreas, Population};
use rand::seq::IteratorRandom;
use crate::civilization::city_construction::city_construction_events::{BuildCityCommand, EndPlayerCityConstruction};
use crate::civilization::movement::movement_events::{MoveTokenFromAreaToAreaCommand, PlayerMovementEnded};
use crate::civilization::population_expansion::population_expansion_events::ExpandPopulationManuallyCommand;

pub struct StupidAiPlugin;

impl Plugin for StupidAiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<StupidAiEvent>()
            .add_event::<SelectStupidMove>()
            .add_systems(Update, (
                setup_stupid_ai.run_if(in_state(GameState::Playing)),
                select_stupid_move.run_if(in_state(GameState::Playing)),
            ))
            .observe(on_add_available_moves)
        ;
    }
}

pub fn on_add_available_moves(
    trigger: Trigger<OnAdd, AvailableMoves>,
    mut event_writer: EventWriter<SelectStupidMove>,
) {
    event_writer.send(SelectStupidMove::new(trigger.entity()));
}

#[derive(Event, Debug, Reflect)]
pub struct SelectStupidMove {
    pub player: Entity,
}

impl SelectStupidMove {
    pub fn new(player: Entity) -> Self {
        SelectStupidMove { player }
    }
}

#[derive(Event, Debug, Reflect)]
pub struct StupidAiEvent {
    pub player: Entity,
}

impl StupidAiEvent {
    pub fn new(player: Entity) -> Self {
        StupidAiEvent { player }
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct StupidAi;

fn setup_stupid_ai(
    mut stupid_ai_event: EventReader<StupidAiEvent>,
    mut commands: Commands,
) {
    for e in stupid_ai_event.read() {
        commands.entity(e.player).insert(StupidAi::default());
    }
}

fn select_stupid_move(
    mut event_reader: EventReader<SelectStupidMove>,
    player_moves: Query<(&AvailableMoves, &PlayerAreas)>,
    _area_query: Query<(&Population, Has<BuiltCity>)>,
    mut expand_writer: EventWriter<ExpandPopulationManuallyCommand>,
    mut move_tokens_writer: EventWriter<MoveTokenFromAreaToAreaCommand>,
    mut end_movement_writer: EventWriter<PlayerMovementEnded>,
    mut build_city_writer: EventWriter<BuildCityCommand>,
    mut end_player_city_construction: EventWriter<EndPlayerCityConstruction>,
) {
    for event in event_reader.read() {
        if let Ok((available_moves, _player_areas)) = player_moves.get(event.player) {
            /*
            So, the moves will always really be of maximum one or two types (for now). 
            One is ending the moves, like, not building a city or something like that.
            The other is simply to do something. So, for our first iteration of all this, 
            the stupid stupid AI will always do a random move.
             */
            let mut rng = rand::thread_rng();
            if let Some(selected_move) = available_moves.moves.values().into_iter().choose(&mut rng) {
                match selected_move {
                    Move::PopulationExpansion(pop_exp_move) => {
                        expand_writer.send(ExpandPopulationManuallyCommand::new(event.player, pop_exp_move.area, pop_exp_move.max_tokens));
                    }
                    Move::Movement(movement_move) => {
                        //A little complexity here: If possible, leave two, but also, always make a move
                        match movement_move.max_tokens {
                            1 => {
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 1, event.player));
                            }
                            2 => {
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 1, event.player));
                            }
                            _ => {
                                move_tokens_writer.send(MoveTokenFromAreaToAreaCommand::new(movement_move.source, movement_move.target, 2, event.player));
                            }
                        }
                    }
                    Move::EndMovement => {
                        end_movement_writer.send(PlayerMovementEnded::default());
                    }
                    Move::CityConstruction(build_city_move) => {
                       build_city_writer.send(BuildCityCommand::new(event.player, build_city_move.target)); 
                    }
                    Move::EndCityConstruction => {
                        end_player_city_construction.send(EndPlayerCityConstruction::new(event.player));
                    }
                }
            }
        }
    }
    /*
    If an area that the player has a presence in contains more than max
    population in that area, move surplus population to other areas,
    prioritizing empty areas. 
    
    This is a bit more advanced than the algorithm below and gives us 
    opportunity to add some more components to areas and players to
    make all this easier.
    
    Players can have a list of "I have a presence in the following 
    areas" to simplify iterating over them
    Areas in turn might have a "I am empty"-component to make it 
    easy to exclude or include empty areas in queries.
     */
}