//! What a supervisor (the `civ_lobby` service, see docs/online-games-plan.md)
//! needs to know about this game, and when it should go away.
//!
//! - `GET /api/status` (served by `http.rs`) returns a snapshot that the game
//!   thread refreshes every frame: seats, joins, connections, started/finished.
//! - `EXIT_AFTER_GAME_OVER_SECS`: when set, the process exits that many
//!   seconds after the game reaches `GameOver`, so the supervisor can reap
//!   it. The delay gives clients time to see the final scores. Unset = run
//!   forever (the old behaviour).

use crate::game::Seats;
use adv_civ::civilization::GameInfoAndStuff;
use adv_civ::{GameActivity, GameState};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub total_players: usize,
    pub human_seats: usize,
    /// Seats someone has claimed at least once (a name is bound).
    pub joined: usize,
    /// Seats with a live connection right now.
    pub connected: usize,
    pub started: bool,
    pub finished: bool,
    pub round: usize,
    pub phase: Option<String>,
}

impl StatusSnapshot {
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "total_players": self.total_players,
            "human_seats": self.human_seats,
            "joined": self.joined,
            "connected": self.connected,
            "started": self.started,
            "finished": self.finished,
            "round": self.round,
            "phase": self.phase,
        })
        .to_string()
    }
}

/// Shared between the game thread (writer) and the HTTP thread (reader).
#[derive(Resource, Clone, Default)]
pub struct SharedStatus(pub Arc<Mutex<StatusSnapshot>>);

/// Total player count, fixed at boot (`HeadlessGamePlugin` reads the env).
#[derive(Resource, Clone, Copy)]
pub struct TotalPlayers(pub usize);

#[derive(Resource)]
struct ExitAt(Instant);

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SharedStatus>()
            .add_systems(Update, (refresh_status, exit_when_due))
            .add_systems(OnEnter(GameActivity::GameOver), schedule_exit);
    }
}

fn refresh_status(
    shared: Res<SharedStatus>,
    seats: Res<Seats>,
    total: Res<TotalPlayers>,
    game_state: Res<State<GameState>>,
    activity: Option<Res<State<GameActivity>>>,
    game_info: Option<Res<GameInfoAndStuff>>,
) {
    let activity = activity.map(|a| a.get().clone());
    let snapshot = StatusSnapshot {
        total_players: total.0,
        human_seats: seats.0.len(),
        joined: seats.0.iter().filter(|s| s.name.is_some()).count(),
        connected: seats.0.iter().filter(|s| s.client.is_some()).count(),
        started: *game_state.get() == GameState::Playing,
        finished: activity == Some(GameActivity::GameOver),
        round: game_info.map_or(0, |g| g.round),
        phase: activity.map(|a| format!("{a:?}")),
    };
    if let Ok(mut current) = shared.0.lock()
        && *current != snapshot
    {
        *current = snapshot;
    }
}

pub fn exit_delay() -> Option<Duration> {
    std::env::var("EXIT_AFTER_GAME_OVER_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
}

fn schedule_exit(mut commands: Commands) {
    if let Some(delay) = exit_delay() {
        info!("Game over — exiting in {}s (EXIT_AFTER_GAME_OVER_SECS)", delay.as_secs());
        commands.insert_resource(ExitAt(Instant::now() + delay));
    }
}

fn exit_when_due(exit_at: Option<Res<ExitAt>>, mut exit: MessageWriter<AppExit>) {
    if exit_at.is_some_and(|at| Instant::now() >= at.0) {
        info!("Exiting after game over");
        exit.write(AppExit::Success);
    }
}
