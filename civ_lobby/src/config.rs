//! Lobby configuration, all from env so the Docker image needs no config file.

use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    /// Port the lobby listens on (API + proxy). `LOBBY_PORT`, default 8080.
    pub port: u16,
    /// The game server binary. `GAME_BIN`, default `adv_civ_server` next to
    /// this executable.
    pub game_bin: PathBuf,
    /// One sub-directory per game (its working directory: save file, log).
    /// `GAMES_DIR`, default `games`.
    pub games_dir: PathBuf,
    /// Game `i` gets ports `base + 2i` (WebSocket) and `base + 2i + 1` (HTTP).
    /// `GAME_PORT_BASE`, default 6000. These stay inside the container.
    pub port_base: u16,
    /// `MAX_GAMES`, default 10.
    pub max_games: usize,
    /// Creates allowed per client IP per hour. `MAX_CREATES_PER_HOUR`, default 6.
    pub max_creates_per_hour: usize,
    /// What clients dial, handed to every game as `PUBLIC_ADDR` (netcode
    /// tokens are minted for it). `PUBLIC_ADDR`, default `127.0.0.1:<port>`.
    pub public_addr: String,
    /// WebSocket base URL; a game's socket is `<base>/g/<code>/ws`.
    /// `PUBLIC_WS_BASE`, e.g. `wss://civ.kidvhs.com`; default `ws://127.0.0.1:<port>`.
    pub public_ws_base: String,
    /// Site URL for join links (`<url>/?game=<code>`). `PUBLIC_URL`, optional:
    /// without it the client builds the link from its own origin.
    pub public_url: Option<String>,
    /// A game nobody has started is stopped after this. `UNSTARTED_TTL_HOURS`, default 24.
    pub unstarted_ttl: Duration,
    /// A started game with no human connected is stopped after this.
    /// `IDLE_TTL_HOURS`, default 168 (a week).
    pub idle_ttl: Duration,
    /// Passed to each game as `EXIT_AFTER_GAME_OVER_SECS`. Default 900.
    pub exit_after_game_over_secs: u64,
}

fn env<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

impl Config {
    pub fn from_env() -> Self {
        let port: u16 = env("LOBBY_PORT", 8080);
        let game_bin = std::env::var("GAME_BIN").map_or_else(
            |_| {
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|dir| dir.join("adv_civ_server")))
                    .unwrap_or_else(|| PathBuf::from("adv_civ_server"))
            },
            PathBuf::from,
        );
        Config {
            port,
            game_bin,
            games_dir: env("GAMES_DIR", PathBuf::from("games")),
            port_base: env("GAME_PORT_BASE", 6000),
            max_games: env("MAX_GAMES", 10),
            max_creates_per_hour: env("MAX_CREATES_PER_HOUR", 6),
            public_addr: std::env::var("PUBLIC_ADDR")
                .unwrap_or_else(|_| format!("127.0.0.1:{port}")),
            public_ws_base: std::env::var("PUBLIC_WS_BASE").map_or_else(
                |_| format!("ws://127.0.0.1:{port}"),
                |s| s.trim_end_matches('/').to_string(),
            ),
            public_url: std::env::var("PUBLIC_URL")
                .ok()
                .map(|s| s.trim_end_matches('/').to_string()),
            unstarted_ttl: hours(env("UNSTARTED_TTL_HOURS", 24)),
            idle_ttl: hours(env("IDLE_TTL_HOURS", 168)),
            exit_after_game_over_secs: env("EXIT_AFTER_GAME_OVER_SECS", 900),
        }
    }
}
