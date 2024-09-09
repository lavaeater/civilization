use bevy::prelude::{Commands, Entity, EventWriter, NextState, Query, ResMut, With};
use bevy::core::Name;
use bevy_console::PrintConsoleLine;
use clap::builder::StyledStr;
use crate::civilization::conflict::conflict_components::UnresolvedConflict;
use crate::civilization::general::general_components::Population;
use crate::civilization::general::general_events::ReturnTokenToStock;
use crate::GameActivity;

pub fn resolve_conflicts(
    mut conflict_zones: Query<(Entity, &Name, &mut Population), With<UnresolvedConflict>>,
    mut return_token: EventWriter<ReturnTokenToStock>,
    mut commands: Commands,
    name_query: Query<&Name>,
    mut write_line: EventWriter<PrintConsoleLine>,
) {
    for (area_entity, name, mut population) in conflict_zones.iter_mut() {
        let temp_map = population.player_tokens.clone();
        let mut players = temp_map.keys().map(|k| *k).collect::<Vec<Entity>>();
        players.sort_by(|a, b| temp_map[b].len().cmp(&temp_map[a].len()));

        let all_lengths_equal = players.iter().all(|player| {
            temp_map[player].len() == temp_map[&players[0]].len()
        });

        while population.total_population() > population.max_population {
            write_line.send(
                PrintConsoleLine::new(
                    StyledStr::from(
                        format!("Resolving conflict in: {:?}, total: {}, max: {}",
                                name,
                                population.total_population(),
                                population.max_population)
                    )
                )
            );
            if all_lengths_equal {
                write_line.send(
                    PrintConsoleLine::new(
                        StyledStr::from("All players have same amount of tokens")
                    )
                );
                for player in players.iter() {
                    let p_name = name_query.get(*player).unwrap();
                    write_line.send(
                        PrintConsoleLine::new(
                            StyledStr::from(
                                format!("Removing token for player: {:?}, Population now: {}",
                                        p_name,
                                        population.total_population())
                            )
                        )
                    );
                    if let Some(tokens) = population.player_tokens.get_mut(player) {
                        if let Some(token_entity) = tokens.pop() {
                            write_line.send(
                                PrintConsoleLine::new(
                                    StyledStr::from("Sending back token!")
                                )
                            );
                            return_token.send(ReturnTokenToStock { token_entity });
                        }
                    }
                }
            } else {
                // pop the last player (has least tokens on this area)
                let player = players.pop().unwrap();
                if let Some(vec) = population.player_tokens.get_mut(&player) {
                    if let Some(token_entity) = vec.pop() {
                        return_token.send(ReturnTokenToStock { token_entity });
                    } else {
                        // has no more tokens here, remove altogether
                        population.player_tokens.remove(&player);
                    }
                }
                // insert the player back at the start of the line... yihaa.
                players.insert(0, player);
            }
        }
        commands.entity(area_entity).remove::<UnresolvedConflict>();
    }
}

pub fn find_conflict_zones(
    pop_query: Query<(Entity, &Name, &Population)>,
    mut commands: Commands,
    mut write_line: EventWriter<PrintConsoleLine>,
    mut next_state: ResMut<NextState<GameActivity>>,
) {
    pop_query.iter().filter(|(_, _, pop)| {
        pop.is_conflict_zone()
    }).for_each(|(conflict_zone, name, _)| {
        write_line.send(PrintConsoleLine::new(StyledStr::from(format!("Conflict zone found: {:?}", name))));
        commands.entity(conflict_zone).insert(UnresolvedConflict);
    });
    next_state.set(GameActivity::CityConstruction);
}