use bevy::app::{Plugin, Update};
use bevy::prelude::{in_state, App, Commands, Component, IntoSystemConfigs, OnEnter, Reflect};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::GameState;

pub struct StupidAiPlugin;

impl Plugin for StupidAiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), setup_stupid_ai)
            .add_systems(Update, (
                move_tokens.run_if(in_state(GameActivity::Movement)),
                build_cities.run_if(in_state(GameActivity::CityConstruction)),
                remove_surplus.run_if(in_state(GameActivity::RemoveSurplusPopulation)),
                reduce_cities.run_if(in_state(GameActivity::CheckCitySupport)),
            ))
        ;
    }

}

#[derive(Component, Debug, Reflect)]
pub struct StupidAi;


// Run different systems depending on what game state we are in...
fn setup_stupid_ai(
    mut commands: Commands,
) {
    
}

fn move_tokens() {
    
}

fn build_cities() {
    
}

fn remove_surplus() {
    
}

fn reduce_cities() {
    
}