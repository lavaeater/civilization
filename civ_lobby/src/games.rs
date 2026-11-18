//! The game registry: spawning one `adv_civ_server` per game, tracking its
//! status, and reaping it when it ends or is abandoned.
//!
//! The registry lives in memory only. The game server can't yet resume a
//! networked game from its save (seats would need re-binding), so a lobby
//! restart ends every running game; see docs/online-play-progress.md.

use crate::codes;
use crate::config::Config;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// What the game server reports on `GET /api/status` (`adv_civ_server/src/status.rs`).
#[derive(Clone, Debug, Default, Deserialize, serde::Serialize)]
pub struct GameStatus {
    pub total_players: usize,
    pub human_seats: usize,
    pub joined: usize,
    pub connected: usize,
    pub started: bool,
    pub finished: bool,
    pub round: usize,
    pub phase: Option<String>,
}

pub struct Game {
    pub code: String,
    pub total_players: usize,
    pub human_seats: usize,
    /// Index into the port pool: WS port `base + 2*slot`, HTTP port `+1`.
    pub slot: usize,
    pub ws_port: u16,
    pub http_port: u16,
    pub created: Instant,
    pub child: Mutex<Child>,
    pub state: Mutex<Tracked>,
}

#[derive(Default)]
pub struct Tracked {
    pub status: Option<GameStatus>,
    /// Last time a human was connected (or the game was created).
    pub last_active: Option<Instant>,
}

#[derive(Debug)]
pub enum CreateError {
    BadSettings(String),
    Full,
    RateLimited,
    SpawnFailed(String),
}

pub struct Lobby {
    pub config: Config,
    pub games: Mutex<HashMap<String, Arc<Game>>>,
    creates: Mutex<HashMap<IpAddr, Vec<Instant>>>,
    pub http: reqwest::Client,
}

impl Lobby {
    pub fn new(config: Config) -> Self {
        Lobby {
            config,
            games: Mutex::default(),
            creates: Mutex::default(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
        }
    }

    pub async fn get(&self, code: &str) -> Option<Arc<Game>> {
        self.games.lock().await.get(code).cloned()
    }

    pub async fn create(
        &self,
        total_players: usize,
        human_seats: usize,
        ip: IpAddr,
    ) -> Result<Arc<Game>, CreateError> {
        validate(total_players, human_seats)?;
        self.check_rate(ip).await?;

        let game = {
            let mut games = self.games.lock().await;
            if games.len() >= self.config.max_games {
                return Err(CreateError::Full);
            }
            let used: Vec<usize> = games.values().map(|g| g.slot).collect();
            let slot = (0..self.config.max_games)
                .find(|s| !used.contains(s) && self.ports_free(*s))
                .ok_or(CreateError::Full)?;
            let code = loop {
                let code = codes::generate(&mut rand::rng());
                if !games.contains_key(&code) {
                    break code;
                }
            };
            let game = Arc::new(self.spawn(code, total_players, human_seats, slot)?);
            games.insert(game.code.clone(), game.clone());
            game
        };

        if let Err(e) = self.wait_until_up(&game).await {
            self.remove(&game.code).await;
            return Err(CreateError::SpawnFailed(e));
        }
        info!(
            code = %game.code,
            total_players, human_seats, ws_port = game.ws_port,
            "game created"
        );
        Ok(game)
    }

    fn ports(&self, slot: usize) -> (u16, u16) {
        // `max_games` is small; the pool fits in u16 for any sane base.
        let ws = self.config.port_base + u16::try_from(slot * 2).unwrap_or(u16::MAX - 1);
        (ws, ws + 1)
    }

    /// A port can still be held by a game that just exited, or by
    /// something else entirely; skip slots we can't bind.
    fn ports_free(&self, slot: usize) -> bool {
        let (ws, http) = self.ports(slot);
        [ws, http]
            .iter()
            .all(|p| std::net::TcpListener::bind(("0.0.0.0", *p)).is_ok())
    }

    fn spawn(
        &self,
        code: String,
        total_players: usize,
        human_seats: usize,
        slot: usize,
    ) -> Result<Game, CreateError> {
        let (ws_port, http_port) = self.ports(slot);
        let dir = self.config.games_dir.join(&code);
        std::fs::create_dir_all(&dir)
            .map_err(|e| CreateError::SpawnFailed(format!("game dir {}: {e}", dir.display())))?;
        let log = std::fs::File::create(dir.join("server.log"))
            .map_err(|e| CreateError::SpawnFailed(format!("log file: {e}")))?;
        let log_err = log
            .try_clone()
            .map_err(|e| CreateError::SpawnFailed(format!("log file: {e}")))?;

        let child = Command::new(&self.config.game_bin)
            .current_dir(&dir) // savegame.json lands in the game's own directory
            .env("SEATS", human_seats.to_string())
            .env("NUM_PLAYERS", total_players.to_string())
            .env("PORT", ws_port.to_string())
            .env("HTTP_PORT", http_port.to_string())
            .env("NETCODE_KEY", "random") // a token for one game is useless in another
            .env("PUBLIC_ADDR", &self.config.public_addr)
            .env(
                "PUBLIC_WS",
                format!("{}/g/{code}/ws", self.config.public_ws_base),
            )
            .env(
                "EXIT_AFTER_GAME_OVER_SECS",
                self.config.exit_after_game_over_secs.to_string(),
            )
            // The lobby never serves the client through a game process.
            .env("CLIENT_DIR", "/nonexistent")
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(log_err)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                CreateError::SpawnFailed(format!("{}: {e}", self.config.game_bin.display()))
            })?;

        Ok(Game {
            code,
            total_players,
            human_seats,
            slot,
            ws_port,
            http_port,
            created: Instant::now(),
            child: Mutex::new(child),
            state: Mutex::new(Tracked {
                status: None,
                last_active: Some(Instant::now()),
            }),
        })
    }

    async fn wait_until_up(&self, game: &Game) -> Result<(), String> {
        for _ in 0..60 {
            if let Some(status) = self.fetch_status(game).await {
                game.state.lock().await.status = Some(status);
                return Ok(());
            }
            if let Ok(Some(exit)) = game.child.lock().await.try_wait() {
                return Err(format!("game server exited during start-up: {exit}"));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err("game server did not come up within 30 s".into())
    }

    pub async fn fetch_status(&self, game: &Game) -> Option<GameStatus> {
        let url = format!("http://127.0.0.1:{}/api/status", game.http_port);
        let response = self.http.get(url).send().await.ok()?;
        let bytes = response.bytes().await.ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    async fn check_rate(&self, ip: IpAddr) -> Result<(), CreateError> {
        let mut creates = self.creates.lock().await;
        let hour_ago = Instant::now().checked_sub(Duration::from_secs(3600));
        let recent = creates.entry(ip).or_default();
        recent.retain(|t| hour_ago.is_none_or(|h| *t > h));
        if recent.len() >= self.config.max_creates_per_hour {
            return Err(CreateError::RateLimited);
        }
        recent.push(Instant::now());
        Ok(())
    }

    pub async fn remove(&self, code: &str) {
        let game = self.games.lock().await.remove(code);
        if let Some(game) = game {
            let mut child = game.child.lock().await;
            if let Ok(None) = child.try_wait() {
                let _ = child.kill().await;
            }
        }
    }

    /// One pass of the reaper: refresh every game's status and drop the
    /// ones that exited or were abandoned.
    pub async fn reap(&self) {
        let games: Vec<Arc<Game>> = self.games.lock().await.values().cloned().collect();
        for game in games {
            if let Some(reason) = self.reap_reason(&game).await {
                info!(code = %game.code, reason, "removing game");
                self.remove(&game.code).await;
            }
        }
    }

    async fn reap_reason(&self, game: &Game) -> Option<&'static str> {
        match game.child.lock().await.try_wait() {
            Ok(Some(_)) => return Some("game server exited"),
            Ok(None) => {}
            Err(e) => warn!(code = %game.code, "try_wait failed: {e}"),
        }
        let status = self.fetch_status(game).await;
        let mut tracked = game.state.lock().await;
        let now = Instant::now();
        if let Some(status) = status {
            if status.connected > 0 {
                tracked.last_active = Some(now);
            }
            tracked.status = Some(status);
        }
        let started = tracked.status.as_ref().is_some_and(|s| s.started);
        let idle_for = now.duration_since(tracked.last_active.unwrap_or(game.created));
        if !started && now.duration_since(game.created) > self.config.unstarted_ttl {
            Some("never started")
        } else if started && idle_for > self.config.idle_ttl {
            Some("no players connected for too long")
        } else {
            None
        }
    }

    pub async fn shutdown(&self) {
        let codes: Vec<String> = self.games.lock().await.keys().cloned().collect();
        for code in codes {
            self.remove(&code).await;
        }
    }
}

fn validate(total_players: usize, human_seats: usize) -> Result<(), CreateError> {
    if !(2..=9).contains(&total_players) {
        return Err(CreateError::BadSettings(
            "total_players must be between 2 and 9".into(),
        ));
    }
    if human_seats == 0 || human_seats > total_players {
        return Err(CreateError::BadSettings(
            "human_seats must be between 1 and total_players".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_validated() {
        assert!(validate(2, 1).is_ok());
        assert!(validate(9, 9).is_ok());
        assert!(validate(1, 1).is_err());
        assert!(validate(10, 2).is_err());
        assert!(validate(5, 0).is_err());
        assert!(validate(3, 4).is_err());
    }
}
