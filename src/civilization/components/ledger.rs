//! Diagnostics for AI tuning: what each player lost, and in which phase.
//!
//! Every population token that leaves the board goes through
//! `on_add_return_token_to_stock`, and every city that leaves it either
//! through `on_add_return_city_to_stock` or [`record_city_loss`]. Those record
//! here, keyed by the `GameActivity` that was running, and `log_round_state`
//! prints one `[LOSS]` line per player per round and then clears the ledger.
//! Headless self-play (`scripts/selfplay.py`) adds those lines up per faction.

use crate::GameActivity;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Entity, Resource, State, World};

#[derive(Resource, Default, Debug)]
pub struct LossLedger {
    /// Board tokens returned to stock, per (player, phase).
    pub tokens: HashMap<(Entity, GameActivity), usize>,
    /// Treasury tokens returned to stock (taxes spent), per (player, phase).
    pub treasury: HashMap<(Entity, GameActivity), usize>,
    /// Cities removed from the board, per (player, phase).
    pub cities: HashMap<(Entity, GameActivity), usize>,
    /// Cities taken over from another player (Civil War, Treachery, tax
    /// revolts), per (player, phase): city gains that are not construction.
    pub cities_gained: HashMap<(Entity, GameActivity), usize>,
}

impl LossLedger {
    /// One player's entries in `map`, as `Phase=n` sorted by phase name.
    pub fn summary(map: &HashMap<(Entity, GameActivity), usize>, player: Entity) -> String {
        let mut parts: Vec<String> = map
            .iter()
            .filter(|((p, _), n)| *p == player && **n > 0)
            .map(|((_, activity), n)| format!("{activity:?}={n}"))
            .collect();
        parts.sort();
        parts.join(",")
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
        self.treasury.clear();
        self.cities.clear();
        self.cities_gained.clear();
    }
}

type LedgerColumn = fn(&mut LossLedger) -> &mut HashMap<(Entity, GameActivity), usize>;

/// Counts one more `player` entry in a ledger column under the current phase.
/// Does nothing in apps (tests, mostly) that have no ledger or no game state.
pub fn record_loss(commands: &mut Commands, player: Entity, column: LedgerColumn) {
    commands.queue(move |world: &mut World| record_in(world, player, column));
}

fn record_in(world: &mut World, player: Entity, pick: LedgerColumn) {
    let Some(activity) = world
        .get_resource::<State<GameActivity>>()
        .map(|s| s.get().clone())
    else {
        return;
    };
    if let Some(mut ledger) = world.get_resource_mut::<LossLedger>() {
        *pick(&mut ledger).entry((player, activity)).or_insert(0) += 1;
    }
}

/// Records a city `player` took over from someone else.
pub fn record_city_gain(commands: &mut Commands, player: Entity) {
    record_loss(commands, player, |l| &mut l.cities_gained);
}

/// Records a city `player` lost, for code paths that retire city tokens
/// themselves instead of going through `ReturnCityToStock`.
pub fn record_city_loss(commands: &mut Commands, player: Entity) {
    record_loss(commands, player, |l| &mut l.cities);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn losses_are_counted_per_player_and_phase() {
        let mut world = World::new();
        world.init_resource::<LossLedger>();
        world.insert_resource(State::new(GameActivity::Conflict));
        let (a, b) = (
            Entity::from_raw_u32(1).unwrap(),
            Entity::from_raw_u32(2).unwrap(),
        );
        record_in(&mut world, a, |l| &mut l.tokens);
        record_in(&mut world, a, |l| &mut l.tokens);
        record_in(&mut world, b, |l| &mut l.cities);

        let ledger = world.resource::<LossLedger>();
        assert_eq!(LossLedger::summary(&ledger.tokens, a), "Conflict=2");
        assert_eq!(LossLedger::summary(&ledger.tokens, b), "");
        assert_eq!(LossLedger::summary(&ledger.cities, b), "Conflict=1");
    }

    #[test]
    fn without_a_game_state_nothing_is_recorded() {
        let mut world = World::new();
        world.init_resource::<LossLedger>();
        record_in(&mut world, Entity::PLACEHOLDER, |l| &mut l.tokens);
        assert!(world.resource::<LossLedger>().tokens.is_empty());
    }
}
