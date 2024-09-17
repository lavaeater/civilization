use bevy_console::ConsoleCommand;
use bevy::prelude::{Entity, EventWriter, NextState, Query, ResMut, With};
use bevy::core::Name;
use clap::Parser;
use crate::civilization::general::general_components::{Faction, StartArea};
use crate::civilization::general::general_events::MoveTokensFromStockToAreaCommand;
use crate::GameActivity;
use crate::player::Player;

#[derive(Parser, ConsoleCommand)]
#[command(name = "start")]
pub struct StartCommand;

pub fn start_command(
    mut command: ConsoleCommand<StartCommand>,
    player_query: Query<(Entity, &Name, &Faction), With<Player>>,
    start_area_query: Query<(Entity, &Name, &StartArea)>,
    mut writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if let Some(Ok(StartCommand {})) = command.take() {
        for (player_entity, name, player_faction) in player_query.iter() {
            if let Some((area_entity, area_name, _)) = start_area_query.iter().find(|(_, _, start_area)| start_area.faction == player_faction.faction) {
                writer.send(
                    MoveTokensFromStockToAreaCommand {
                        area_entity,
                        player_entity,
                        number_of_tokens: 1,
                    });
                command.reply(format!("{:?} adds a token to {:?}!", name, area_name));
            }
        }
        next_state.set(GameActivity::PopulationExpansion);
    }
}