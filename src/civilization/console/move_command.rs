use clap::Parser;
use bevy_console::ConsoleCommand;
use bevy::prelude::{Entity, EventWriter, Query, Res, With};
use bevy::core::Name;
use crate::civilization::census::census_resources::GameInfoAndStuff;
use crate::civilization::general::general_components::GameArea;
use crate::civilization::movement::movement_components::MoveableTokens;
use crate::civilization::movement::movement_events::MoveTokenFromAreaToAreaCommand;

#[derive(Parser, ConsoleCommand)]
#[command(name = "move")]
pub struct MoveCommand {
    from: String,
    to: String,
    number_of_tokens: usize,
}

pub fn perform_move(
    mut command: ConsoleCommand<MoveCommand>,
    source_query: Query<(Entity, &Name, &MoveableTokens)>,
    target_query: Query<(Entity, &Name), With<GameArea>>,
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
                        source_area: source_entity,
                        target_area: target_entity,
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