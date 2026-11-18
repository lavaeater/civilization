//! `civ_lobby`: hosts many online games at once (docs/online-games-plan.md).
//!
//! Each game is its own `adv_civ_server` process. The lobby creates them,
//! reports their status, proxies `/g/<code>/api/*` (HTTP) and `/g/<code>/ws`
//! (the game WebSocket) to the right one, and stops them when they end or
//! are abandoned.
//!
//! | Route | Does |
//! |---|---|
//! | `POST /api/games` `{total_players, human_seats}` | create a game → `{code, api_base, ws_url, join_url?}` |
//! | `GET /api/games/<code>` | its settings + live status, or 404 |
//! | `ANY /g/<code>/api/*` | proxied to that game's HTTP API (`/api/join`, …) |
//! | `GET /g/<code>/ws` | WebSocket proxied to that game |
//! | `GET /api/health` | lobby liveness |

mod codes;
mod config;
mod games;
mod proxy;

use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use games::{CreateError, Lobby};
use serde::Deserialize;
use serde_json::json;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

type AppState = Arc<Lobby>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = config::Config::from_env();
    info!(?config, "civ_lobby starting");
    if !config.game_bin.is_file() {
        error!(
            "game server binary not found at {} (set GAME_BIN)",
            config.game_bin.display()
        );
    }
    let port = config.port;
    let lobby = Arc::new(Lobby::new(config));

    // Reaper: refresh statuses and stop finished/abandoned games.
    let reaper = lobby.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        loop {
            tick.tick().await;
            reaper.reap().await;
        }
    });

    let app = router(lobby.clone());
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap_or_else(|e| panic!("cannot bind lobby port {port}: {e}"));
    info!("listening on 0.0.0.0:{port}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("server error");

    info!("shutting down: stopping every game");
    lobby.shutdown().await;
}

fn router(lobby: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { Json(json!({ "ok": true })) }))
        .route("/api/games", post(create_game))
        .route("/api/games/{code}", get(game_status))
        .route("/g/{code}/api/{*rest}", any(proxy::http))
        .route("/g/{code}/ws", get(proxy::websocket))
        .with_state(lobby)
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = ctrl_c.await;
}

#[derive(Deserialize)]
struct CreateGame {
    total_players: usize,
    human_seats: usize,
}

pub fn error(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(json!({ "error": message.into() }))).into_response()
}

/// The client's IP for rate limiting: the first `X-Forwarded-For` hop when
/// behind the proxies (pingora → Caddy), else the socket peer.
fn client_ip(headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| peer.ip())
}

async fn create_game(
    State(lobby): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<CreateGame>,
) -> Response {
    let ip = client_ip(&headers, peer);
    match lobby.create(body.total_players, body.human_seats, ip).await {
        Ok(game) => {
            let code = &game.code;
            Json(json!({
                "code": code,
                "api_base": format!("/g/{code}"),
                "ws_url": format!("{}/g/{code}/ws", lobby.config.public_ws_base),
                "join_url": lobby.config.public_url.as_ref().map(|u| format!("{u}/?game={code}")),
                "total_players": game.total_players,
                "human_seats": game.human_seats,
            }))
            .into_response()
        }
        Err(CreateError::BadSettings(msg)) => error(StatusCode::BAD_REQUEST, msg),
        Err(CreateError::Full) => error(
            StatusCode::SERVICE_UNAVAILABLE,
            "the server is hosting as many games as it can; try again later",
        ),
        Err(CreateError::RateLimited) => error(
            StatusCode::TOO_MANY_REQUESTS,
            "too many games created from here recently",
        ),
        Err(CreateError::SpawnFailed(msg)) => {
            error!("could not start a game: {msg}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "could not start the game server")
        }
    }
}

async fn game_status(State(lobby): State<AppState>, Path(code): Path<String>) -> Response {
    let Some(code) = codes::normalize(&code) else {
        return error(StatusCode::NOT_FOUND, "no such game");
    };
    let Some(game) = lobby.get(&code).await else {
        return error(StatusCode::NOT_FOUND, "no such game");
    };
    // Fresh status if the game answers, else the last one the reaper saw.
    let status = match lobby.fetch_status(&game).await {
        Some(status) => Some(status),
        None => game.state.lock().await.status.clone(),
    };
    Json(json!({
        "code": game.code,
        "total_players": game.total_players,
        "human_seats": game.human_seats,
        "status": status,
    }))
    .into_response()
}
