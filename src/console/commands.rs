use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::StartArea;
use crate::civilization::general::plugin::MoveTokensFromStockToAreaCommand;
use crate::player::Player;
use bevy::app::{App, Plugin};
use bevy::prelude::{Entity, EventWriter, NextState, Query, ResMut, With};
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
        // .add_console_command::<ListMoves, _>(list_moves)
        // .add_console_command::<MoveCommand, _>(perform_move)
        ;
    }
}

// #[derive(Parser, ConsoleCommand)]
// #[command(name = "move")]
// struct MoveCommand {
//     from: String,
//     to: String,
//     number_of_tokens: usize,
// }
//
// fn perform_move(
//     mut command: ConsoleCommand<MoveCommand>,
//     moveable_tokens: Query<(&Name, &MoveableTokens)>,
//     area_query: Query<(&Name, &Children), With<Area>>,
//     population_query: Query<&Population>,
//     parent_query: Query<&Parent>,
//     mut move_command: EventWriter<MoveTokenFromAreaToAreaCommand>,
// ) {
//     if let Some(Ok(MoveCommand { from: source, to: target, number_of_tokens })) = command.take() {
//         let source_name = Name::from(source);
//         let target_name = Name::from(target);
//         if let Some((_, tokens)) = moveable_tokens.iter().find(|(name, _)| *name == &source_name) {
//             if tokens.tokens.len() >= number_of_tokens {
//                 let from_pop_entity = parent_query.get(tokens.tokens[0]).unwrap().get();
//                 if let Some((_, children)) = area_query.iter().find(|(name, _)| *name == &target_name) {
//                     let to_pop_entity = children.iter().find(|child| population_query.contains(**child)).unwrap();
//                     move_command.send(MoveTokenFromAreaToAreaCommand {
//                         from_area_population: from_pop_entity,
//                         to_area_population: *to_pop_entity,
//                         tokens: tokens.tokens.iter().take(number_of_tokens).copied().collect(),
//                     });
//                 } else {
//                     command.reply(format!("Could not find target area: {}", target_name));
//                 }
//             } else {
//                 command.reply(format!("Not enough tokens in {}", source_name));
//             }
//         } else {
//             command.reply(format!("Could not find source area: {}", source_name));
//         }
//     }
// }
//
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "moves")]
// struct ListMoves;
//
// fn list_moves(
//     mut command: ConsoleCommand<ListMoves>,
//     moveable_tokens: Query<(&Name, &MoveableTokens)>,
//     name_query: Query<&Name>,
//     game_info: Res<GameInfoAndStuff>,
//     mut clear_moves: EventWriter<ClearMovesCommand>
// ) {
//     if let Some(Ok(ListMoves {})) = command.take() {
//         if let Some(_player_to_move) = game_info.current_mover {
//             let moves = moveable_tokens
//                 .iter()
//                 .map(|(from_name, move_specs)| {
//                     (from_name, move_specs.tokens.iter().count(),
//                      move_specs
//                          .targets
//                          .iter()
//                          .map(|target| {
//                              let target_name = name_query.get(*target).unwrap();
//                              target_name
//                          }).collect::<Vec<&Name>>()
//                     )
//                 });
//
//             let message = moves.map(|(from_name, number_of_tokens, targets)| {
//                 format!("{from_name} can move max {number_of_tokens} to: {:?}", targets.iter().map(|name| name.as_str()).collect::<Vec<&str>>().join(", "))
//             }).collect::<Vec<String>>().join("\n");
//             clear_moves.send(ClearMovesCommand {});
//             command.reply(format!("Moves: {}", message));
//         }
//     }
// }

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
