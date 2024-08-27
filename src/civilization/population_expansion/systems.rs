use bevy::prelude::{Commands, Entity, EventWriter, NextState, Query, ResMut, With, Without};
use crate::civilization::game_phases::game_activity::GameActivity;
use crate::civilization::general::components::Population;
use crate::civilization::general::events::MoveTokensFromStockToAreaCommand;
use crate::civilization::general::components::Stock;
use crate::civilization::population_expansion::components::{ExpandAutomatically, ExpandManually, NeedsExpansion};

pub fn check_population_expansion_eligibility(
    area_pop_query: Query<(Entity, &Population), Without<NeedsExpansion>>,
    player_stock_query: Query<(Entity, &Stock), Without<NeedsExpansion>>,
    mut commands: Commands,
) {
    for (player, stock) in player_stock_query.iter() {
        // how many tokens has the player?
        let tokens_in_stock = stock.tokens.len();

        let mut required_tokens = 0;
        for (pop_entity, pop) in area_pop_query.iter() {
            if let Some(p) = pop.player_tokens.get(&player) {
                let rt = match p.len() {
                    1 => { 1 }
                    0 => { 0 }
                    _ => { 2 }
                };
                if rt > 0 {
                    commands.entity(pop_entity).insert(NeedsExpansion {});
                }

                required_tokens += rt;
            };
        }
        if required_tokens > 0 {
            if required_tokens <= tokens_in_stock {
                commands
                    .entity(player)
                    .insert((ExpandAutomatically { required_tokens }, NeedsExpansion {}));
            } else {
                commands
                    .entity(player)
                    .insert((ExpandManually {}, NeedsExpansion {}));
            }
        }
    }
}

pub fn expand_population(
    area_query: Query<(Entity, &Population), With<NeedsExpansion>>,
    to_expand: Query<(Entity, &ExpandAutomatically), With<NeedsExpansion>>,
    mut event_writer: EventWriter<MoveTokensFromStockToAreaCommand>,
    mut next_state: ResMut<NextState<GameActivity>>,
    mut commands: Commands,
) {
    for (pop_entity, pop) in area_query.iter() {
        for (player, tokens) in pop.player_tokens.iter() {
            if to_expand.contains(*player) {
                match tokens.len() {
                    0 => {}
                    1 => {
                        event_writer.send(MoveTokensFromStockToAreaCommand {
                            area_entity: pop_entity,
                            player_entity: *player,
                            number_of_tokens: 1,
                        });
                    }
                    _ => {
                        event_writer.send(MoveTokensFromStockToAreaCommand {
                            area_entity: pop_entity,
                            player_entity: *player,
                            number_of_tokens: 2,
                        });
                    }
                }
            }
        }
        commands.entity(pop_entity).remove::<NeedsExpansion>();
    }
    for (player, _) in to_expand.iter() {
        commands.entity(player).remove::<NeedsExpansion>();
        commands.entity(player).remove::<ExpandAutomatically>();
    }
    next_state.set(GameActivity::Census);
}