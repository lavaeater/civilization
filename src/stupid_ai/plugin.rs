use bevy::app::{Plugin, Update};
use bevy::prelude::{in_state, App, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Reflect};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::GameState;

pub struct StupidAiPlugin;

impl Plugin for StupidAiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<StupidAiEvent>()
            .add_systems(Update, (
                setup_stupid_ai.run_if(in_state(GameState::Playing)),
                move_tokens.run_if(in_state(GameActivity::Movement)),
                build_cities.run_if(in_state(GameActivity::CityConstruction)),
                remove_surplus.run_if(in_state(GameActivity::RemoveSurplusPopulation)),
                reduce_cities.run_if(in_state(GameActivity::CheckCitySupport)),
            ))
        ;
    }

}

#[derive(Event, Debug, Reflect)]
pub struct StupidAiEvent {
    pub player: Entity
}

#[derive(Component, Debug, Reflect, Default)]
pub struct StupidAi;


// Run different systems depending on what game state we are in...
fn setup_stupid_ai(
    mut stupid_ai_event: EventReader<StupidAiEvent>,
    mut commands: Commands,
) {
    for e in stupid_ai_event.read() {
        commands.entity(e.player).insert(StupidAi::default());
    }
}

fn move_tokens(
    game_info_and_stuff:
) {
    if let Some(_player_to_move) = game_info.current_mover {
        let moves = moveable_tokens
            .iter()
            .map(|(from_name, move_specs)| {
                (from_name, move_specs.tokens.iter().count(),
                 move_specs
                     .targets
                     .iter()
                     .map(|target| {
                         let target_name = name_query.get(*target).unwrap();
                         target_name
                     }).collect::<Vec<&Name>>()
                )
            });

        let message = moves.map(|(from_name, number_of_tokens, targets)| {
            format!("{from_name} can move max {number_of_tokens} to: {:?}", targets.iter().map(|name| name.as_str()).collect::<Vec<&str>>().join(", "))
        }).collect::<Vec<String>>().join("\n");
}

fn build_cities() {
    
}

fn remove_surplus() {
    
}

fn reduce_cities() {
    
}