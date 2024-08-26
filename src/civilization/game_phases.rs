use bevy::app::Update;
use bevy::prelude::{in_state, Event, EventReader, EventWriter, IntoSystemConfigs, Plugin, Reflect};
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::GameState;

pub struct GamePhasesPlugin;

impl Plugin for GamePhasesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_event::<GameActivityStarted>()
            .add_event::<GameActivityEnded>()
            .add_systems(
                Update, (
                    print_names_of_phases.run_if(in_state(GameState::Playing)),
                    direct_game_phases.run_if(in_state(GameState::Playing)),
                ),
            )
        ;
    }
}

pub fn print_names_of_phases(
    mut write_line: EventWriter<PrintConsoleLine>,
    mut activity_start: EventReader<GameActivityStarted>,
    mut activity_end: EventReader<GameActivityEnded>,
) {
    for activity in activity_start.read() {
        let a = activity.0;
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Started: {:?}", a))));
    }

    for activity in activity_end.read() {
        let a = activity.0;
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Ended: {:?}", a))));
    }
}

pub fn direct_game_phases(
    mut activity_end: EventReader<GameActivityEnded>,
    mut activity_start: EventWriter<GameActivityStarted>,
) {
    for activity in activity_end.read() {
        match activity.0 {
            GameActivity::CollectTaxes => {
                activity_start.send(GameActivityStarted(GameActivity::PopulationExpansion));
            }
            GameActivity::PopulationExpansion => {
                activity_start.send(GameActivityStarted(GameActivity::Census));
            }
            GameActivity::Census => {
                activity_start.send(GameActivityStarted(GameActivity::Movement));
            }
            GameActivity::ShipConstruction => {}
            GameActivity::Movement => {}
            GameActivity::Conflict => {}
            GameActivity::CityConstruction => {}
            GameActivity::RemoveSurplusPopulation => {}
            GameActivity::CheckCitySupport => {}
            GameActivity::AcquireTradeCards => {}
            GameActivity::Trade => {}
            GameActivity::ResolveCalamities => {}
            GameActivity::AcquireCivilizationCards => {}
            GameActivity::MoveSuccessionMarkers => {}
        }
    }
}

#[derive(Debug, Reflect, PartialEq, Copy, Clone)]
pub enum GameActivity {
    CollectTaxes,
    PopulationExpansion,
    Census,
    ShipConstruction,
    Movement,
    Conflict,
    CityConstruction,
    RemoveSurplusPopulation,
    CheckCitySupport,
    AcquireTradeCards,
    Trade,
    ResolveCalamities,
    AcquireCivilizationCards,
    MoveSuccessionMarkers,
}

#[derive(Event, Debug, Reflect)]
pub struct GameActivityStarted(pub GameActivity);

#[derive(Event, Debug, Reflect)]
pub struct GameActivityEnded(pub GameActivity);