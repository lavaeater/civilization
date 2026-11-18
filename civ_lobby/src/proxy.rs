//! Forwarding a game's traffic to its process: plain HTTP for the join API,
//! and a WebSocket bridge for the game connection itself (lightyear sends
//! binary frames; we copy them both ways untouched).

use crate::{AppState, codes, error};
use axum::body::Bytes;
use axum::extract::ws::{self, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;
use tracing::{debug, warn};

pub async fn http(
    State(lobby): State<AppState>,
    Path((code, rest)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(game) = find(&lobby, &code).await else {
        return error(StatusCode::NOT_FOUND, "no such game");
    };
    let url = format!("http://127.0.0.1:{}/api/{rest}", game.http_port);
    let Ok(method) = reqwest::Method::from_bytes(method.as_str().as_bytes()) else {
        return error(StatusCode::METHOD_NOT_ALLOWED, "unsupported method");
    };
    let mut request = lobby.http.request(method, url).body(body);
    if let Some(ct) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
    {
        request = request.header("content-type", ct);
    }
    match request.send().await {
        Ok(response) => {
            let status =
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();
            let body = response.bytes().await.unwrap_or_default();
            (status, [(header::CONTENT_TYPE, content_type)], body).into_response()
        }
        Err(e) => {
            warn!(code = %game.code, "proxy to game failed: {e}");
            error(StatusCode::BAD_GATEWAY, "the game server is not answering")
        }
    }
}

pub async fn websocket(
    State(lobby): State<AppState>,
    Path(code): Path<String>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let Some(game) = find(&lobby, &code).await else {
        return error(StatusCode::NOT_FOUND, "no such game");
    };
    let port = game.ws_port;
    let code = game.code.clone();
    upgrade.on_upgrade(move |client| async move {
        let url = format!("ws://127.0.0.1:{port}/ws");
        match tokio_tungstenite::connect_async(url).await {
            Ok((upstream, _)) => bridge(client, upstream).await,
            Err(e) => warn!(%code, "cannot reach the game's WebSocket: {e}"),
        }
        debug!(%code, "websocket closed");
    })
}

async fn find(lobby: &AppState, code: &str) -> Option<std::sync::Arc<crate::games::Game>> {
    lobby.get(&codes::normalize(code)?).await
}

type Upstream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn bridge(client: WebSocket, upstream: Upstream) {
    let (mut client_tx, mut client_rx) = client.split();
    let (mut up_tx, mut up_rx) = upstream.split();

    let to_game = async {
        while let Some(Ok(msg)) = client_rx.next().await {
            let msg = match msg {
                ws::Message::Binary(b) => tungstenite::Message::Binary(b),
                ws::Message::Text(t) => tungstenite::Message::text(t.as_str()),
                ws::Message::Ping(p) => tungstenite::Message::Ping(p),
                ws::Message::Pong(p) => tungstenite::Message::Pong(p),
                ws::Message::Close(_) => break,
            };
            if up_tx.send(msg).await.is_err() {
                break;
            }
        }
        let _ = up_tx.close().await;
    };
    let to_client = async {
        while let Some(Ok(msg)) = up_rx.next().await {
            let msg = match msg {
                tungstenite::Message::Binary(b) => ws::Message::Binary(b),
                tungstenite::Message::Text(t) => ws::Message::text(t.as_str()),
                tungstenite::Message::Ping(p) => ws::Message::Ping(p),
                tungstenite::Message::Pong(p) => ws::Message::Pong(p),
                tungstenite::Message::Close(_) => break,
                tungstenite::Message::Frame(_) => continue,
            };
            if client_tx.send(msg).await.is_err() {
                break;
            }
        }
        let _ = client_tx.close().await;
    };
    // Whichever side hangs up first ends the bridge.
    tokio::select! {
        () = to_game => {},
        () = to_client => {},
    }
}
