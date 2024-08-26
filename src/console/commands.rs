use crate::civilization::census::GameInfoAndStuff;
use crate::civilization::civ::{MoveTokensFromStockToAreaCommand, Population, StartArea};
use crate::civilization::movement::MoveableTokens;
use crate::player::Player;
use bevy::app::{App, Plugin};
use bevy::prelude::{Children, Entity, EventWriter, Name, Parent, Query, Res, With};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use clap::Parser;
use crate::civilization::game_phases::{GameActivity, GameActivityEnded};

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
            .add_console_command::<MoveCommand, _>(perform_move)
        ;
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "move")]
struct MoveCommand {
    from: String,
    to: String,
    number_of_tokens: usize,
}

fn perform_move(
    mut command: ConsoleCommand<MoveCommand>,
) {
     if let Some(Ok(MoveCommand { from, to, number_of_tokens })) = command.take() {
        command.reply(format!("Moving {} tokens from {} to {}", number_of_tokens, from, to));
    }}

#[derive(Parser, ConsoleCommand)]
#[command(name = "moves")]
struct ListMoves;

fn list_moves(
    mut command: ConsoleCommand<ListMoves>,
    moveable_tokens: Query<(&Name, &MoveableTokens)>,
    name_query: Query<&Name>,
    game_info: Res<GameInfoAndStuff>,
) {
    if let Some(Ok(ListMoves {})) = command.take() {
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
            command.reply(format!("Moves: {}", message));
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "popexp")]
struct ExpandPopulation;

fn expand_population(
    mut command: ConsoleCommand<ExpandPopulation>,
    mut writer: EventWriter<GameActivityEnded>,
) {
    if let Some(Ok(ExpandPopulation {})) = command.take() {
        writer.send(GameActivityEnded(GameActivity::CollectTaxes));
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
    mut writer: EventWriter<MoveTokensFromStockToAreaCommand>,
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