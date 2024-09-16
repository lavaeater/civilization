use bevy_console::ConsoleCommand;
use bevy::prelude::{Entity, EventWriter, Query, With};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::city_construction::city_construction_components::CityBuildTargets;
use crate::civilization::city_construction::city_construction_events::BuildCity;
use crate::player::Player;

#[derive(Parser, ConsoleCommand)]
#[command(name = "bc")]
pub struct BuildCityCommand {
    player: String,
    area: String,
}

pub fn build_city(
    mut command: ConsoleCommand<BuildCityCommand>,
    player_query: Query<(Entity, &Name, &CityBuildTargets), With<Player>>,
    name_query: Query<&Name>,
    mut build_city: EventWriter<BuildCity>,
) {
    if let Some(Ok(BuildCityCommand { player, area })) = command.take() {
        if let Some((player_entity, targets)) = player_query.iter().find(|(_, name, _)| **name == Name::from(player.clone())).map(|(entity, _, targets)| (entity, targets)) {
            if let Some(target_entity) = targets.targets.iter().filter(|t| *name_query.get(**t).unwrap() == Name::from(area.clone())).next() {
                build_city.send(BuildCity {
                    player: player_entity,
                    area: *target_entity,
                });
                command.reply("Building City");
            } else {
                command.reply("Could not find target area");
            }
        }
    }
}