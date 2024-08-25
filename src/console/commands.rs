use bevy::app::{App, Plugin};
use bevy::prelude::{Children, Entity, EventWriter, HierarchyQueryExt, Name, Parent, Query, Res, With};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use clap::Parser;
use itertools::Itertools;
use crate::civilization::census::GameInfoAndStuff;
use crate::civilization::civ::{Area, GameActivity, GameActivityStarted, LandPassage, MoveTokensFromStockToAreaCommand, Population, StartArea, Token};
use crate::civilization::movement::TokenCanMove;
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
            .add_console_command::<ListMoves, _>(list_moves)
        ;
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "moves")]
struct ListMoves;

fn list_moves(
    mut command: ConsoleCommand<ListMoves>,
    moveable_tokens: Query<(Entity, &Token), With<TokenCanMove>>,
    parent: Query<&Parent>,
    area_query: Query<(&Area, &LandPassage)>,
    name_query: Query<&Name>,
    game_info: Res<GameInfoAndStuff>
) {
    if let Some(Ok(ListMoves {})) = command.take() {
        if let Some(player_to_move) = game_info.current_mover {
            let message = moveable_tokens
                .iter()
                .filter(|(_, t)|{
                    t.player == player_to_move
            }).map(|(token_entity, token)| {
                //find area, is top entity
                let area_entity: Entity = *parent.iter_ancestors(token_entity).filter(|e| {
                    area_query.contains(*e)
                }).collect::<Vec<Entity>>().first().unwrap();

                if let Ok(n) = name_query.get(area_entity) {
                    if let Ok((p, lp)) = area_query.get(area_entity) {
                        let lands = lp
                            .to_areas
                            .iter()
                            .map(|targets| { name_query.get(*targets).unwrap() })
                            .collect();
                        format!("Can move from {n} to {:?}", lands)
                    }
                }
            }).join("\n");
            command.reply(message);
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "popexp")]
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