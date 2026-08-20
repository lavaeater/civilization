//! HTTP endpoint for the invite-link join flow (docs/multiplayer.md): a
//! client POSTs a name and gets back a short-lived netcode `ConnectToken`
//! plus the WebSocket URL. The token is minted with the server's private
//! key, which never leaves the server.
//!
//! It also serves the wasm web client (the `trunk build` output) as static
//! files, so a single command-line server is enough to play: open
//! `http://<host>:<HTTP_PORT>/` and the browser gets the client, calls
//! `/api/join` same-origin, and connects to the WebSocket URL the server
//! advertises (see `PUBLIC_WS`). Behind Caddy the static serving is unused
//! (Caddy serves `dist/` itself) but harmless.
//!
//! Runs `tiny_http` in a thread (same pattern as the agent API) and bridges
//! join requests into the ECS through a channel, so seat checks and
//! client-id assignment happen on the game thread.

use crate::game::Seats;
use base64::Engine;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lightyear::netcode::{ConnectToken, Key, generate_key};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::time::Duration;

pub use adv_civ_protocol::PROTOCOL_ID;

pub fn http_port() -> u16 {
    std::env::var("HTTP_PORT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(5112)
}

/// The address clients are told to connect to (and that tokens are minted
/// for). Override with PUBLIC_ADDR when the server sits behind NAT/Docker.
pub fn public_addr() -> String {
    std::env::var("PUBLIC_ADDR").unwrap_or_else(|_| format!("127.0.0.1:{}", crate::server_port()))
}

/// The WebSocket URL handed to joining clients. Behind the Caddy front this
/// is `wss://<domain>/ws`; the default suits direct localhost connections.
pub fn public_ws_url() -> String {
    std::env::var("PUBLIC_WS").unwrap_or_else(|_| format!("ws://{}", public_addr()))
}

/// Directory of static web-client files to serve (the `trunk build` output).
/// Missing directory = API only (e.g. behind Caddy, which serves it instead).
pub fn client_dir() -> std::path::PathBuf {
    std::env::var("CLIENT_DIR")
        .unwrap_or_else(|_| "dist".into())
        .into()
}

/// Netcode credentials. The key comes from NETCODE_KEY (64 hex chars), or
/// `random` to generate one at boot, or defaults to the all-zero dev key so
/// manually-authenticated dev clients keep working.
#[derive(Resource, Clone)]
pub struct NetcodeKeys {
    pub key: Key,
    pub protocol_id: u64,
}

impl NetcodeKeys {
    pub fn from_env() -> Self {
        let key = match std::env::var("NETCODE_KEY").ok().as_deref().map(str::trim) {
            Some("random") => {
                info!("NETCODE_KEY=random — generated a boot-time private key");
                generate_key()
            }
            Some(hex) => match parse_hex_key(hex) {
                Some(key) => key,
                None => {
                    error!("NETCODE_KEY is not 64 hex chars — falling back to the dev key");
                    Key::default()
                }
            },
            None => {
                warn!("No NETCODE_KEY set — using the all-zero dev key (fine for localhost only)");
                Key::default()
            }
        };
        NetcodeKeys {
            key,
            protocol_id: PROTOCOL_ID,
        }
    }
}

fn parse_hex_key(hex: &str) -> Option<Key> {
    if hex.len() != 64 {
        return None;
    }
    let mut key = Key::default();
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(key)
}

struct JoinRequest {
    name: String,
    reply: SyncSender<JoinReply>,
}

enum JoinReply {
    Ok { token_b64: String, client_id: u64 },
    Full,
    Error(String),
}

#[derive(Resource)]
struct HttpJoinRequests(Mutex<Receiver<JoinRequest>>);

/// Names registered via HTTP, waiting for their netcode connection to show
/// up: client_id → player name. Drained by the seat-claiming system.
#[derive(Resource, Default)]
pub struct PendingJoins(pub HashMap<u64, String>);

pub struct HttpApiPlugin;

impl Plugin for HttpApiPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = std::sync::mpsc::channel::<JoinRequest>();
        let port = http_port();
        std::thread::spawn(move || serve(tx, port));
        app.insert_resource(NetcodeKeys::from_env())
            .insert_resource(HttpJoinRequests(Mutex::new(rx)))
            .init_resource::<PendingJoins>()
            .add_systems(Update, process_join_requests);
        info!("HTTP join API on port {port}");
    }
}

/// The HTTP side: parse, hand over to the ECS, wait briefly for the answer.
fn serve(tx: Sender<JoinRequest>, port: u16) {
    let server = match tiny_http::Server::http(("0.0.0.0", port)) {
        Ok(server) => server,
        Err(e) => {
            error!("HTTP API failed to bind port {port}: {e}");
            return;
        }
    };
    let dir = client_dir();
    if dir.join("index.html").is_file() {
        info!("Serving web client from {} on port {port}", dir.display());
    } else {
        info!(
            "No web client at {} (CLIENT_DIR) — HTTP API only. \
             Run `trunk build --release` or point CLIENT_DIR at the dist dir.",
            dir.display()
        );
    }
    for mut request in server.incoming_requests() {
        let method = request.method().as_str().to_string();
        let url = request.url().to_string();
        let respond = |request: tiny_http::Request, status: u16, body: String| {
            let content_type =
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .expect("static header");
            // Dev convenience: lets a trunk-served (different-origin) web
            // client call the API. Production is same-origin behind Caddy.
            let cors =
                tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..])
                    .expect("static header");
            let _ = request.respond(
                tiny_http::Response::from_string(body)
                    .with_status_code(status)
                    .with_header(content_type)
                    .with_header(cors),
            );
        };

        match (method.as_str(), url.as_str()) {
            ("OPTIONS", _) => {
                let _ = request.respond(
                    tiny_http::Response::empty(204)
                        .with_header(
                            tiny_http::Header::from_bytes(
                                &b"Access-Control-Allow-Origin"[..],
                                &b"*"[..],
                            )
                            .expect("static header"),
                        )
                        .with_header(
                            tiny_http::Header::from_bytes(
                                &b"Access-Control-Allow-Headers"[..],
                                &b"content-type"[..],
                            )
                            .expect("static header"),
                        ),
                );
            }
            ("POST", "/api/join") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let name = serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| v["name"].as_str().map(str::to_string))
                    .unwrap_or_default();
                if name.is_empty() {
                    respond(request, 400, r#"{"error":"missing name"}"#.into());
                    continue;
                }
                let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
                if tx
                    .send(JoinRequest {
                        name,
                        reply: reply_tx,
                    })
                    .is_err()
                {
                    respond(request, 500, r#"{"error":"server shutting down"}"#.into());
                    continue;
                }
                match reply_rx.recv_timeout(Duration::from_secs(2)) {
                    Ok(JoinReply::Ok {
                        token_b64,
                        client_id,
                    }) => {
                        let body = serde_json::json!({
                            "connect_token": token_b64,
                            "client_id": client_id,
                            "ws_url": public_ws_url(),
                            "protocol_id": PROTOCOL_ID,
                        });
                        respond(request, 200, body.to_string());
                    }
                    Ok(JoinReply::Full) => {
                        respond(request, 409, r#"{"error":"all seats taken"}"#.into())
                    }
                    Ok(JoinReply::Error(e)) => {
                        respond(request, 500, serde_json::json!({ "error": e }).to_string())
                    }
                    Err(_) => respond(request, 504, r#"{"error":"game thread busy"}"#.into()),
                }
            }
            ("GET", "/api/health") => respond(request, 200, r#"{"ok":true}"#.into()),
            // Anything else that is a GET: try the static web client.
            ("GET", path) => serve_static(request, path),
            _ => respond(request, 404, r#"{"error":"not found"}"#.into()),
        }
    }
}

/// Serve a file from the web-client directory, falling back to `index.html`
/// for unknown paths (single-page app style). Read-only GET, path-traversal
/// guarded.
fn serve_static(request: tiny_http::Request, url: &str) {
    let not_found = |request: tiny_http::Request| {
        let _ =
            request.respond(tiny_http::Response::from_string("not found").with_status_code(404));
    };

    let path = url.split('?').next().unwrap_or("/");
    let rel = path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    // No `..` segments may escape the client directory.
    if rel.split('/').any(|seg| seg == "..") {
        return not_found(request);
    }

    let dir = client_dir();
    let mut full = dir.join(rel);
    if !full.is_file() {
        full = dir.join("index.html"); // SPA fallback
    }
    match std::fs::File::open(&full) {
        Ok(file) => {
            let header =
                tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type_for(&full))
                    .expect("valid content-type header");
            let _ = request.respond(tiny_http::Response::from_file(file).with_header(header));
        }
        Err(_) => not_found(request),
    }
}

fn content_type_for(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("webmanifest") => "application/manifest+json",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        _ => "application/octet-stream",
    }
}

/// The ECS side: check seat availability, mint the token, register the
/// pending join so the netcode connection can claim its seat by client id.
fn process_join_requests(
    requests: Res<HttpJoinRequests>,
    seats: Res<Seats>,
    keys: Res<NetcodeKeys>,
    mut pending: ResMut<PendingJoins>,
    mut next_client_id: Local<u64>,
) {
    let Ok(requests) = requests.0.lock() else {
        return;
    };
    while let Ok(request) = requests.try_recv() {
        let free_seats = seats.0.iter().filter(|s| s.client.is_none()).count();
        if free_seats <= pending.0.len() {
            let _ = request.reply.send(JoinReply::Full);
            continue;
        }

        *next_client_id += 1;
        let client_id = 1_000_000 + *next_client_id;
        let reply = match mint_token(&keys, client_id) {
            Ok(token_b64) => {
                info!(
                    "Minted ConnectToken for {} (client id {client_id})",
                    request.name
                );
                pending.0.insert(client_id, request.name.clone());
                JoinReply::Ok {
                    token_b64,
                    client_id,
                }
            }
            Err(e) => JoinReply::Error(e),
        };
        let _ = request.reply.send(reply);
    }
}

fn mint_token(keys: &NetcodeKeys, client_id: u64) -> Result<String, String> {
    // lightyear's address parsing doesn't do DNS; resolve PUBLIC_ADDR here.
    // (Over WebSocket the address is informational — the ws URL dials.)
    use std::net::ToSocketAddrs;
    let public = public_addr();
    let addr = public
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .ok_or_else(|| format!("PUBLIC_ADDR {public:?} does not resolve"))?;
    let token = ConnectToken::build(addr, keys.protocol_id, client_id, keys.key)
        .expire_seconds(60)
        .generate()
        .map_err(|e| format!("token generation failed: {e}"))?;
    let bytes = token
        .try_into_bytes()
        .map_err(|e| format!("token serialization failed: {e}"))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}
