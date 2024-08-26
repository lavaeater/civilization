use bevy::prelude::{EventReader, EventWriter};
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::game_phases::plugin::{GameActivity, GameActivityEnded, GameActivityStarted};

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