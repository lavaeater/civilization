use clap::Parser;
use bevy_console::ConsoleCommand;
use bevy::prelude::{Entity, EventWriter, Query};
use bevy::core::Name;
use crate::civilization::city_support::city_support_events::EliminateCity;
use crate::civilization::general::general_components::BuiltCity;

#[derive(Parser, ConsoleCommand)]
#[command(name = "ec")]
pub struct EliminateCityCommand {
    pub area_name: String,
}

pub fn eliminate_city(
    mut command: ConsoleCommand<EliminateCityCommand>,
    area_query: Query<(Entity, &Name, &BuiltCity)>,
    mut eliminate_city: EventWriter<EliminateCity>,
) {
    if let Some(Ok(EliminateCityCommand { area_name })) = command.take() {
        if let Some((area_entity, _, built_city)) = area_query
            .iter()
            .find(|(_, name, _)| **name == Name::from(area_name.clone()))
        {
            eliminate_city.send(EliminateCity {
                city: built_city.city,
                area_entity,
            });
        }
    }
}