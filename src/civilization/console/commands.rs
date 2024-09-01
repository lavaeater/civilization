use crate::civilization::census::resources::GameInfoAndStuff;
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::{Area, BuiltCity, CitySite, Faction, Population, StartArea};
use crate::civilization::general::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::movement::components::MoveableTokens;
use crate::civilization::movement::events::{ClearAllMoves, MoveTokenFromAreaToAreaCommand};
use crate::player::Player;
use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Entity, EventWriter, Has, Name, NextState, Query, Res, ResMut, With};
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use clap::Parser;
use crate::civilization::city_construction::components::{CityBuildTargets, DoneBuilding};
use crate::civilization::city_construction::events::BuildCity;
use crate::civilization::city_support::plugin::EliminateCity;

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
            .add_console_command::<ShowBoardCommand, _>(show_board)
            .add_console_command::<ListBuildsCommand, _>(list_builds)
            .add_console_command::<BuildCityCommand, _>(build_city)
            .add_console_command::<PlayerEndBuildingCommand, _>(end_building)
            .add_console_command::<EliminateCityCommand, _>(eliminate_city)
        ;
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "ec")]
struct EliminateCityCommand {
    pub area_name: String,
}

fn eliminate_city(
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

#[derive(Parser, ConsoleCommand)]
#[command(name = "eb")]
struct PlayerEndBuildingCommand {
    pub player_name: String,
}

fn end_building(
    mut command: ConsoleCommand<PlayerEndBuildingCommand>,
    mut commands: Commands,
    player_query: Query<(Entity, &Name)>,
) {
    if let Some(Ok(PlayerEndBuildingCommand { player_name })) = command.take() {
        if let Some(player_entity) = player_query
            .iter()
            .find(|(_, name)| **name == Name::from(player_name.clone()))
            .map(|(entity, _)| entity) {
            commands.entity(player_entity).insert(DoneBuilding {});
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "bc")]
struct BuildCityCommand {
    player: String,
    area: String,
}

fn build_city(
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

#[derive(Parser, ConsoleCommand)]
#[command(name = "builds")]
struct ListBuildsCommand;

fn list_builds(
    mut command: ConsoleCommand<ListBuildsCommand>,
    player_query: Query<(&Name, &CityBuildTargets)>,
    name_query: Query<&Name>,
) {
    if let Some(Ok(ListBuildsCommand {})) = command.take() {
        for (player_name, targets) in player_query.iter() {
            let target_names = targets.targets.iter().map(|target| name_query.get(*target).unwrap().as_str()).collect::<Vec<&str>>().join(", ");
            command.reply(format!("{:?} can build in: {:?}", player_name, target_names));
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "board")]
struct ShowBoardCommand;

fn show_board(
    mut command: ConsoleCommand<ShowBoardCommand>,
    area_query: Query<(&Name, &Population, Has<StartArea>, Has<CitySite>, Has<BuiltCity>)>,
    name_query: Query<&Name>,
) {
    if let Some(Ok(ShowBoardCommand {})) = command.take() {
        for (area_name, population, is_start_area, is_city_site, has_city) in area_query.iter() {
            command.reply(format!("Area: {:?} {:?} has population: {:?}{:?}{:?}", area_name, if is_start_area { "<s>" } else { "" }, population.total_population, if is_city_site { ", City Site" } else { "" }, if has_city { ", Has City" } else { "" }));
            for (player, tokens) in population.player_tokens.iter() {
                command.reply(format!("Player: {:?} has: {:?} tokens", name_query.get(*player).unwrap(), tokens.len()));
            }
        }
    }
}


#[derive(Parser, ConsoleCommand)]
#[command(name = "endmove")]
struct EndMoveCommand;

fn end_move(
    mut command: ConsoleCommand<EndMoveCommand>,
    mut clear_all_moves: EventWriter<ClearAllMoves>,
) {
    if let Some(Ok(EndMoveCommand {})) = command.take() {
        clear_all_moves.send(ClearAllMoves {});
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
