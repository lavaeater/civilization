use clap::Parser;
use bevy_console::ConsoleCommand;
use bevy::prelude::{Has, Query};
use bevy::core::Name;
use crate::civilization::general::general_components::{BuiltCity, CitySite, Population, StartArea};

#[derive(Parser, ConsoleCommand)]
#[command(name = "board")]
pub struct ShowBoardCommand;

pub fn show_board(
    mut command: ConsoleCommand<ShowBoardCommand>,
    area_query: Query<(&Name, &Population, Has<StartArea>, Has<CitySite>, Has<BuiltCity>)>,
    name_query: Query<&Name>,
) {
    if let Some(Ok(ShowBoardCommand {})) = command.take() {
        for (area_name, population, is_start_area, is_city_site, has_city) in area_query.iter() {
            command.reply(format!("Area: {:?} {:?} has population: {:?}{:?}{:?}", area_name, if is_start_area { "<s>" } else { "" }, population.total_population(), if is_city_site { ", City Site" } else { "" }, if has_city { ", Has City" } else { "" }));
            for (player, tokens) in population.player_tokens.iter() {
                command.reply(format!("Player: {:?} has: {:?} tokens", name_query.get(*player).unwrap(), tokens.len()));
            }
        }
    }
}