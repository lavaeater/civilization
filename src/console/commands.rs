use bevy::app::{App, Plugin};
use bevy::prelude::{Children, Entity, EventWriter, Parent, Query, With};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use clap::Parser;
use crate::civilization::civ::{GameActivity, GameActivityStarted, MoveTokensFromStockToAreaCommand, Population, StartArea};
use crate::civilization::population_expansion::CheckPopulationExpansionEligibilityEvent;
use crate::player::Player;

pub struct CommandsPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ConsolePlugin)
            .insert_resource(ConsoleConfiguration {
                // override config here
                ..Default::default()
            })
            .add_console_command::<StartCommand, _>(start_command)
            .add_console_command::<ExpandPopulation, _>(expand_population)
        ;
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "exp-pop")]
struct ExpandPopulation;

fn expand_population(
    mut command: ConsoleCommand<ExpandPopulation>,
    mut writer: EventWriter<GameActivityStarted>
) {
    if let Some(Ok(ExpandPopulation {})) = command.take() {
        writer.send(GameActivityStarted(GameActivity::PopulationExpansion));
        command.reply("We are starting the expansion!")
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "start")]
struct StartCommand;

fn start_command(
    mut command: ConsoleCommand<StartCommand>,
    player_query: Query<Entity, With<Player>>,
    start_area_query: Query<(Entity, &Children), With<StartArea>>,
    pop_query: Query<(Entity, &Parent), With<Population>>,
    mut writer: EventWriter<MoveTokensFromStockToAreaCommand>
) {
    if let Some(Ok(StartCommand {})) = command.take() {
        if let Ok(player_entity) = player_query.get_single() {
            if let Ok((area_entity, children)) = start_area_query.get_single() {
                for child in children {
                    if let Ok((pop_entity, parent)) = pop_query.get(*child) {
                        if parent.get() == area_entity {

                            command.reply("Player adds a token to start area!");
                            writer.send(
                                MoveTokensFromStockToAreaCommand {
                                    population_entity: pop_entity,
                                    stock_entity: player_entity,
                                    number_of_tokens: 1,
                                });
                        }
                    }
                }
            }
        }
    }
}