use crate::civilization::census::resources::GameInfoAndStuff;
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::{Area, StartArea};
use crate::civilization::general::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::movement::components::MoveableTokens;
use crate::civilization::movement::events::{MoveTokenFromAreaToAreaCommand, NextPlayerStarted};
use crate::player::Player;
use bevy::app::{App, Plugin};
use bevy::prelude::{Entity, EventWriter, Name, NextState, Query, Res, ResMut, With};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use clap::Parser;

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
            .add_console_command::<EndMoveCommand, _>(end_move)
        ;
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "endmove")]
struct EndMoveCommand;

fn end_move(
    mut command: ConsoleCommand<EndMoveCommand>,
    mut next_player_started: EventWriter<NextPlayerStarted>
) {
    if let Some(Ok(EndMoveCommand {})) = command.take() {
        next_player_started.send(NextPlayerStarted {});
        command.reply("Next player started!");
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
    source_query: Query<(Entity, &Name, &MoveableTokens)>,
    target_query: Query<(Entity, &Name), With<Area>>,
    game_info_and_stuff: Res<GameInfoAndStuff>,
    mut move_command: EventWriter<MoveTokenFromAreaToAreaCommand>,
) {
    if let Some(Ok(MoveCommand { from: source, to: target, number_of_tokens })) = command.take() {
        let source_name = Name::from(source);
        let target_name = Name::from(target);
        if let Some((source_entity, _, tokens)) = source_query.iter().find(|(_, name, _)| *name == &source_name) {
            if let Some((target_entity, _)) = target_query.iter().find(|(_, name)| *name == &target_name) {
                if tokens.tokens.len() >= number_of_tokens {
                    move_command.send(MoveTokenFromAreaToAreaCommand {
                        source_entity,
                        target_entity,
                        number_of_tokens,
                        player: game_info_and_stuff.current_mover.unwrap(),
                    });
                } else {
                    command.reply(format!("Not enough tokens in {}", source_name));
                }
            } else {
                command.reply(format!("Could not find target area: {}", target_name));
            }
        } else {
            command.reply(format!("Could not find source area: {}", source_name));
        }
    }
}

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
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    if let Some(Ok(ExpandPopulation {})) = command.take() {
        next_state.set(GameActivity::PopulationExpansion);
        command.reply("We are starting the expansion!")
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "start")]
struct StartCommand;

fn start_command(
    mut command: ConsoleCommand<StartCommand>,
    player_query: Query<Entity, With<Player>>,
    start_area_query: Query<Entity, With<StartArea>>,
    mut writer: EventWriter<MoveTokensFromStockToAreaCommand>,
) {
    if let Some(Ok(StartCommand {})) = command.take() {
        if let Ok(player_entity) = player_query.get_single() {
            if let Ok(area_entity) = start_area_query.get_single() {
                command.reply("Player adds a token to start area!");
                writer.send(
                    MoveTokensFromStockToAreaCommand {
                        area_entity,
                        player_entity,
                        number_of_tokens: 1,
                    });
            }
        }
    }
}