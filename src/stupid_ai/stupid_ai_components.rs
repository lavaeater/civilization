use bevy::prelude::*;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct StupidAi;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct IsHuman;

/// A player whose moves come from the external agent API (`AGENT_FACTIONS`).
/// Such players are also `IsHuman` (so the game waits for them) but distinct from
/// a human at the keyboard, so trade/UI logic can route to the API instead.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct AgentControlled;

#[derive(Resource, Default)]
pub struct AiMoveQueue {
    pub pending: Vec<(Entity, f32)>,
}

impl AiMoveQueue {
    pub fn push(&mut self, player: Entity, delay: f32) {
        self.pending.push((player, delay));
    }
}

/// Safety net + diagnostic for the movement phase: counts how many times in a row
/// each AI player has been asked to move without ending. A no-op move (e.g. one the
/// executor silently drops) would otherwise loop forever under a deterministic
/// picker; this trips, logs the offending move, and forces the player to stop.
/// Reset on entering the Movement phase.
#[derive(Resource, Default)]
pub struct MovementLoopGuard {
    pub counts: bevy::platform::collections::HashMap<Entity, u32>,
}

impl MovementLoopGuard {
    /// Warn (with the looping move) once we cross this many consecutive selections.
    pub const WARN_AT: u32 = 80;
    /// Force the player to end movement past this many — a real phase never needs
    /// anywhere near this many selections (≤ token count + a few ferry hops).
    pub const FORCE_END_AT: u32 = 120;
}
