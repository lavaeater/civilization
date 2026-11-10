//! Headless game server (see docs/multiplayer.md).
//!
//! Runs the full, real rules engine — map, all phases, AI opponents — with
//! no rendering, serving human seats over WebSocket. Configuration via env:
//! `SEATS` (human seats, default 2), `NUM_PLAYERS` (total incl. AI,
//! default 5), `PORT` (default 5111), `HTTP_PORT` (default 5112),
//! `EXIT_AFTER_GAME_OVER_SECS` (unset = keep running after the game ends).
//! Saves go to `savegame.json` in the working directory, so a supervisor
//! gives each game its own directory.
//!
//! Also embeds the same agent HTTP API the local single-player build uses
//! (`AGENT_FACTIONS`, see `docs/agent-api-design.md`), on 127.0.0.1:7878 —
//! so a subset of seats can be driven by an external agent instead of a
//! browser or the built-in AI. Keep `AGENT_FACTIONS` disjoint from the
//! reserved `SEATS` factions (see `game.rs::HeadlessGamePlugin`).

mod game;
mod http;
mod net;
mod status;

use adv_civ::GameState;
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use core::time::Duration;
use lightyear::prelude::server::ServerPlugins;

/// Network tick rate. Turn-based game — nothing here is latency-sensitive.
pub const TICK_HZ: f64 = 32.0;

pub fn server_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(5111)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    )
    .add_plugins(LogPlugin::default())
    .add_plugins(StatesPlugin)
    // Loading: wait for RON assets. Menu: lobby. Playing: the game.
    .insert_state(GameState::Loading);

    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.add_plugins(adv_civ_protocol::ProtocolPlugin);

    app.add_plugins((
        game::HeadlessGamePlugin,
        http::HttpApiPlugin,
        net::NetBridgePlugin,
        status::StatusPlugin,
    ));

    app.run();
}
