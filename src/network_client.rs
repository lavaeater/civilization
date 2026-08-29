//! Network client for server-hosted games (docs/multiplayer.md).
//!
//! `GameState::Online` is a parallel top-level flow: the local rules engine
//! (keyed to `GameState::Playing`) stays completely dormant; everything on
//! screen is driven by protocol messages. The server is authoritative — the
//! client renders state and picks from the moves it was offered.
//!
//! First pass: a functional text/button UI (lobby, phase, board summary,
//! moves, hand). The map view comes later.

use crate::GameState;
use adv_civ_protocol::*;
use base64::Engine;
use bevy::prelude::*;
use core::net::SocketAddr;
use core::time::Duration;
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::sync::Mutex;
use std::sync::mpsc;

const TICK_HZ: f64 = 32.0;

/// Where and who. Native reads env (JOIN_URL, SERVER_WS, SERVER_ADDR,
/// PLAYER_NAME); the browser reads the join link's query parameters
/// (`?name=…&api=…&ws=…`), defaulting to same-origin behind Caddy.
#[derive(Resource, Clone)]
pub struct NetworkSettings {
    /// Base URL of the HTTP join API (e.g. `http://127.0.0.1:5112`).
    /// `None` falls back to manual zero-key dev authentication.
    pub api_url: Option<String>,
    /// Explicit WebSocket URL override; otherwise the join response (token
    /// path) or `ws://server_addr` (dev path) decides.
    pub ws_override: Option<String>,
    /// Only used by the manual dev-auth path.
    pub server_addr: SocketAddr,
    pub player_name: String,
}

impl Default for NetworkSettings {
    #[cfg(not(target_family = "wasm"))]
    fn default() -> Self {
        let server_addr = std::env::var("SERVER_ADDR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| "127.0.0.1:5111".parse().expect("valid literal"));
        NetworkSettings {
            api_url: std::env::var("JOIN_URL").ok(),
            ws_override: std::env::var("SERVER_WS").ok(),
            server_addr,
            player_name: std::env::var("PLAYER_NAME").unwrap_or_else(|_| "Newcomer".into()),
        }
    }

    #[cfg(target_family = "wasm")]
    fn default() -> Self {
        let location = web_sys::window().expect("browser window").location();
        let search = location.search().unwrap_or_default();
        let params = web_sys::UrlSearchParams::new_with_str(&search).ok();
        let param = |key: &str| params.as_ref().and_then(|p| p.get(key));
        // The join API is same-origin by default (Caddy proxies /api/*, and
        // the single-binary server serves the client and the API on one
        // port). The WebSocket URL is *not* assumed same-origin: it comes
        // from the join response (the server's PUBLIC_WS) unless the link
        // overrides it with `?ws=`. That way one client build works both
        // behind Caddy (wss://domain/ws) and against a bare server on a LAN
        // (ws://host:5111), decided entirely by server config.
        let origin = location.origin().unwrap_or_default();
        NetworkSettings {
            api_url: Some(param("api").unwrap_or(origin)),
            ws_override: param("ws"),
            server_addr: "127.0.0.1:5111".parse().expect("valid literal"),
            player_name: param("name").unwrap_or_else(|| "Webfriend".into()),
        }
    }
}

/// Successful answer from `POST /api/join`.
pub struct JoinInfo {
    pub token_b64: String,
    pub ws_url: String,
    /// Reconnect secret to save and send on the next join (see
    /// `docs/multiplayer.md`'s session-token section) — always present on a
    /// successful join, whether freshly minted or just the client's own
    /// token handed back.
    pub reconnect_token: Option<String>,
}

/// `localStorage` key for the reconnect secret (web client). Native builds
/// have no persistent storage story here and fall back to `RECONNECT_TOKEN`.
#[cfg(target_family = "wasm")]
const RECONNECT_TOKEN_KEY: &str = "adv_civ_reconnect_token";

#[cfg(target_family = "wasm")]
fn load_reconnect_token() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(RECONNECT_TOKEN_KEY)
        .ok()?
}

#[cfg(target_family = "wasm")]
fn save_reconnect_token(token: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.set_item(RECONNECT_TOKEN_KEY, token);
    }
}

/// Native has no browser storage; a `RECONNECT_TOKEN` env var covers the
/// dev/manual-testing case symmetrically, but there's no way to persist a
/// freshly-minted one across runs beyond logging it for the developer to copy.
#[cfg(not(target_family = "wasm"))]
fn load_reconnect_token() -> Option<String> {
    std::env::var("RECONNECT_TOKEN").ok()
}

#[cfg(not(target_family = "wasm"))]
fn save_reconnect_token(token: &str) {
    info!("[NET] Reconnect token for next run: RECONNECT_TOKEN={token}");
}

/// In-flight token fetch; removed once resolved.
#[derive(Resource)]
struct JoinFetch(Mutex<mpsc::Receiver<Result<JoinInfo, String>>>);

/// True when this session authenticated with a ConnectToken — the seat was
/// already claimed via the HTTP join, so no JoinGame message is sent.
#[derive(Resource, Default)]
struct UsedTokenAuth(bool);

/// Everything the online UI renders, mirrored from server messages.
#[derive(Resource, Default)]
pub struct NetGame {
    pub connected: bool,
    pub seated_as: Option<(String, GameFaction)>,
    pub lobby: Option<LobbyState>,
    pub phase: Option<NetPhase>,
    pub moves: Vec<(usize, NetGameMove)>,
    pub board: Option<GameStateView>,
    pub hand: Vec<(TradeCard, usize)>,
    pub last_error: Option<String>,
    /// First click of a two-area movement move; the second click on a valid
    /// target submits it. Cleared on submit, phase change, or new moves.
    pub selected_source: Option<AreaId>,
    /// UI rebuild flag — set by every mutation above.
    dirty: bool,
}

impl NetGame {
    fn touch(&mut self) {
        self.dirty = true;
    }
}

/// Written by move buttons, drained into the lightyear sender.
#[derive(Message)]
pub struct SubmitNetMove(pub usize);

/// The lightyear client connection entity for this session.
#[derive(Resource)]
struct NetClient(Entity);

#[derive(Component, Default)]
struct OnlineUiRoot;

pub struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
        });
        app.add_plugins(ProtocolPlugin);
        app.init_resource::<NetGame>()
            .insert_resource(NetworkSettings::default())
            .add_message::<SubmitNetMove>()
            .init_resource::<UsedTokenAuth>()
            .init_resource::<NetMapState>()
            .add_systems(OnEnter(GameState::Online), start_join)
            .add_systems(Update, auto_online.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Online), disconnect_and_cleanup)
            .add_systems(
                Update,
                (
                    poll_join_fetch.run_if(resource_exists::<JoinFetch>),
                    join_when_connected,
                    receive_net_messages,
                    forward_submitted_moves,
                    spawn_net_map,
                    handle_map_click,
                    update_net_map_labels,
                    refresh_map_highlights,
                    rebuild_online_ui,
                )
                    .chain()
                    .run_if(in_state(GameState::Online)),
            );
    }
}

/// Dev convenience: AUTO_ONLINE=1 skips the menu straight into Online.
fn auto_online(mut next_state: ResMut<NextState<GameState>>) {
    if std::env::var("AUTO_ONLINE").is_ok_and(|v| !matches!(v.trim(), "" | "0")) {
        next_state.set(GameState::Online);
    }
}

fn start_join(mut commands: Commands, settings: Res<NetworkSettings>, mut net: ResMut<NetGame>) {
    *net = NetGame::default();
    net.touch();

    if let Some(api_url) = settings.api_url.clone() {
        // Join-link flow: fetch a ConnectToken from the HTTP API, connect
        // once it arrives (poll_join_fetch).
        let (tx, rx) = mpsc::channel();
        commands.insert_resource(JoinFetch(Mutex::new(rx)));
        commands.insert_resource(UsedTokenAuth(true));
        info!("Requesting join token from {api_url} …");
        request_join_token(api_url, settings.player_name.clone(), load_reconnect_token(), tx);
    } else {
        // Dev fallback: zero-key manual auth straight at the socket.
        commands.insert_resource(UsedTokenAuth(false));
        let client_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(42, |d| d.as_nanos() as u64);
        let auth = Authentication::Manual {
            server_addr: settings.server_addr,
            client_id,
            private_key: Key::default(),
            protocol_id: PROTOCOL_ID,
        };
        let ws_url = settings
            .ws_override
            .clone()
            .unwrap_or_else(|| format!("ws://{}", settings.server_addr));
        spawn_connection(&mut commands, auth, ws_url, &mut net);
    }
}

/// POST /api/join off the main thread; the result comes back via mpsc.
#[cfg(not(target_family = "wasm"))]
fn request_join_token(
    api_url: String,
    name: String,
    reconnect_token: Option<String>,
    tx: mpsc::Sender<Result<JoinInfo, String>>,
) {
    std::thread::spawn(move || {
        let result = (|| {
            let response = ureq::post(&format!("{api_url}/api/join"))
                .send_string(
                    &serde_json::json!({ "name": name, "token": reconnect_token }).to_string(),
                )
                .map_err(|e| format!("join request failed: {e}"))?;
            let body = response
                .into_string()
                .map_err(|e| format!("join response unreadable: {e}"))?;
            parse_join_response(&body)
        })();
        let _ = tx.send(result);
    });
}

#[cfg(target_family = "wasm")]
fn request_join_token(
    api_url: String,
    name: String,
    reconnect_token: Option<String>,
    tx: mpsc::Sender<Result<JoinInfo, String>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let result = async {
            let response = gloo_net::http::Request::post(&format!("{api_url}/api/join"))
                .body(
                    serde_json::json!({ "name": name, "token": reconnect_token }).to_string(),
                )
                .map_err(|e| format!("join request invalid: {e}"))?
                .send()
                .await
                .map_err(|e| format!("join request failed: {e}"))?;
            let body = response
                .text()
                .await
                .map_err(|e| format!("join response unreadable: {e}"))?;
            parse_join_response(&body)
        }
        .await;
        let _ = tx.send(result);
    });
}

fn parse_join_response(body: &str) -> Result<JoinInfo, String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("join response not JSON: {e}"))?;
    if let Some(error) = value["error"].as_str() {
        return Err(format!("server refused: {error}"));
    }
    Ok(JoinInfo {
        token_b64: value["connect_token"]
            .as_str()
            .ok_or("join response missing connect_token")?
            .to_string(),
        ws_url: value["ws_url"]
            .as_str()
            .ok_or("join response missing ws_url")?
            .to_string(),
        reconnect_token: value["reconnect_token"].as_str().map(str::to_string),
    })
}

fn poll_join_fetch(
    mut commands: Commands,
    fetch: Res<JoinFetch>,
    settings: Res<NetworkSettings>,
    mut net: ResMut<NetGame>,
) {
    let result = match fetch.0.lock() {
        Ok(receiver) => match receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err("join request abandoned".into()),
        },
        Err(_) => Err("join fetch poisoned".into()),
    };
    commands.remove_resource::<JoinFetch>();

    match result {
        Ok(info) => {
            // Keep the local copy current whether it's brand new or just the
            // one we already had handed back — see docs/multiplayer.md.
            if let Some(token) = &info.reconnect_token {
                save_reconnect_token(token);
            }
            let token_bytes = match base64::engine::general_purpose::STANDARD
                .decode(&info.token_b64)
                .map_err(|e| format!("token not base64: {e}"))
            {
                Ok(bytes) => bytes,
                Err(e) => {
                    net.last_error = Some(e);
                    net.touch();
                    return;
                }
            };
            let token = match lightyear::netcode::ConnectToken::try_from_bytes(&token_bytes) {
                Ok(token) => token,
                Err(e) => {
                    net.last_error = Some(format!("invalid ConnectToken: {e}"));
                    net.touch();
                    return;
                }
            };
            // An explicit override (?ws= / SERVER_WS) wins over the
            // server-advertised URL.
            let ws_url = settings.ws_override.clone().unwrap_or(info.ws_url);
            spawn_connection(
                &mut commands,
                Authentication::Token(token),
                ws_url,
                &mut net,
            );
        }
        Err(e) => {
            net.last_error = Some(e);
            net.touch();
        }
    }
}

fn spawn_connection(
    commands: &mut Commands,
    auth: Authentication,
    ws_url: String,
    net: &mut NetGame,
) {
    let Ok(netcode) = NetcodeClient::new(auth, NetcodeConfig::default()) else {
        net.last_error = Some("failed to build netcode client".into());
        net.touch();
        return;
    };
    // Native picks plain-vs-TLS itself; in the browser the ws/wss scheme of
    // the URL decides, and the config type is a unit struct.
    #[cfg(not(target_family = "wasm"))]
    let config = ClientConfig::builder().with_no_encryption();
    #[cfg(target_family = "wasm")]
    let config = ClientConfig;
    let client = commands
        .spawn((
            Client,
            Link::default().with_conditioner(None),
            netcode,
            WebSocketClientIo::from_url(config, ws_url.clone()),
            Name::new("Online game connection"),
        ))
        .id();
    commands.trigger(Connect { entity: client });
    commands.insert_resource(NetClient(client));
    info!("Connecting to {ws_url} …");
}

fn disconnect_and_cleanup(
    mut commands: Commands,
    client: Option<Res<NetClient>>,
    roots: Query<Entity, With<OnlineUiRoot>>,
    map_entities: Query<Entity, Or<(With<NetMapEntity>, With<NetMoveHighlight>)>>,
    mut map_state: ResMut<NetMapState>,
) {
    if let Some(client) = client {
        commands.entity(client.0).despawn();
        commands.remove_resource::<NetClient>();
    }
    commands.remove_resource::<JoinFetch>();
    for root in roots.iter() {
        commands.entity(root).despawn();
    }
    for entity in map_entities.iter() {
        commands.entity(entity).despawn();
    }
    map_state.spawned = false;
    map_state.area_positions.clear();
}

// ───────────────────────────── Map view ─────────────────────────────
//
// The client ships the same map assets as the local game; the board image
// and per-area labels are world-space entities, with the protocol's
// GameStateView projected onto them. View-only: no game logic runs here.

#[derive(Component)]
struct NetMapEntity;

#[derive(Component)]
struct NetMapLabel(AreaId);

/// Marker dot drawn on a currently-clickable area.
#[derive(Component)]
struct NetMoveHighlight;

#[derive(Resource, Default)]
struct NetMapState {
    spawned: bool,
    /// World-space centre of each area, for click hit-testing.
    area_positions: bevy::platform::collections::HashMap<AreaId, Vec2>,
}

/// How close (world units) a click must land to count as hitting an area.
const CLICK_RADIUS: f32 = 35.0;
/// Width of the right-side UI panel; clicks there are not map clicks.
const PANEL_WIDTH: f32 = 460.0;

const MAP_CENTER: Vec3 = Vec3::new(1250.0, 662.5, 0.0);
const MAP_SIZE: Vec2 = Vec2::new(2500.0, 1325.0);

fn spawn_net_map(
    mut commands: Commands,
    maps: Res<Assets<crate::civilization::Map>>,
    textures: Res<crate::loading::TextureAssets>,
    mut state: ResMut<NetMapState>,
    mut camera: Query<(&mut Transform, &mut Projection), With<crate::civilization::GameCamera>>,
    windows: Query<&Window>,
) {
    if state.spawned {
        return;
    }
    // The map RON loads within the first frames; poll until it's there.
    let Some((_, map)) = maps.iter().next() else {
        return;
    };

    commands.spawn((
        NetMapEntity,
        Sprite {
            image: textures.map.clone(),
            ..default()
        },
        Transform::from_xyz(MAP_CENTER.x, MAP_CENTER.y, -1.0),
    ));

    for area in &map.areas {
        state
            .area_positions
            .insert(AreaId(area.id), Vec2::new(area.x, area.y));
        commands.spawn((
            NetMapEntity,
            NetMapLabel(AreaId(area.id)),
            Text2d::new(String::new()),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(area.x, area.y, 5.0),
        ));
    }

    // Frame the whole map (static view; manual camera work comes later).
    if let Ok((mut transform, mut projection)) = camera.single_mut() {
        transform.translation = MAP_CENTER;
        let window_size = windows
            .iter()
            .next()
            .map_or(Vec2::new(1920.0, 1080.0), |w| {
                Vec2::new(w.resolution.width(), w.resolution.height())
            });
        if let Projection::Orthographic(ortho) = &mut *projection {
            ortho.scale = (MAP_SIZE.x / window_size.x).max(MAP_SIZE.y / window_size.y) * 1.05;
        }
    }
    state.spawned = true;
    info!("Online map view ready ({} areas)", map.areas.len());
}

fn update_net_map_labels(net: Res<NetGame>, mut labels: Query<(&NetMapLabel, &mut Text2d)>) {
    if !net.is_changed() {
        return;
    }
    let Some(board) = &net.board else { return };
    let by_id: bevy::platform::collections::HashMap<AreaId, &AreaView> =
        board.areas.iter().map(|view| (view.area, view)).collect();
    for (label, mut text) in &mut labels {
        let Some(view) = by_id.get(&label.0) else {
            continue;
        };
        let mut parts: Vec<String> = Vec::new();
        if let Some(city) = view.city {
            parts.push(format!("[{}]", faction_short(city)));
        }
        parts.extend(
            view.population
                .iter()
                .map(|(faction, count)| format!("{}{count}", faction_short(*faction))),
        );
        text.0 = parts.join(" ");
    }
}

fn faction_short(faction: GameFaction) -> &'static str {
    match faction {
        GameFaction::Egypt => "Eg",
        GameFaction::Crete => "Cr",
        GameFaction::Africa => "Af",
        GameFaction::Asia => "As",
        GameFaction::Assyria => "Ay",
        GameFaction::Babylon => "Ba",
        GameFaction::Illyria => "Il",
        GameFaction::Iberia => "Ib",
        GameFaction::Thrace => "Th",
    }
}

/// The single area a move acts on directly (click it to submit), if any.
fn single_target_area(m: &NetGameMove) -> Option<AreaId> {
    match m {
        NetGameMove::PopulationExpansion { area, .. }
        | NetGameMove::BuildCity { area }
        | NetGameMove::EliminateCity { area, .. } => Some(*area),
        _ => None,
    }
}

/// (source, target) for two-area movement-type moves.
fn move_endpoints(m: &NetGameMove) -> Option<(AreaId, AreaId)> {
    match m {
        NetGameMove::Movement { source, target, .. }
        | NetGameMove::ShipFerry { source, target, .. }
        | NetGameMove::AttackArea { source, target, .. }
        | NetGameMove::AttackCity { source, target, .. } => Some((*source, *target)),
        _ => None,
    }
}

/// What a map click resolves to, given the current moves and selection.
#[derive(Debug, PartialEq, Eq)]
enum ClickOutcome {
    /// Submit this move index.
    Submit(usize),
    /// Update the in-progress movement source (Some) or clear it (None).
    SetSource(Option<AreaId>),
    /// Nothing actionable — leave state untouched.
    Nothing,
}

/// Pure decision for a map click. `clicked == None` means the click missed
/// every area (e.g. empty sea). Kept free of Bevy/IO so it can be unit
/// tested; [`handle_map_click`] does the hit-testing and applies the result.
fn resolve_map_click(
    moves: &[(usize, NetGameMove)],
    selected_source: Option<AreaId>,
    clicked: Option<AreaId>,
) -> ClickOutcome {
    let Some(clicked) = clicked else {
        // A miss only matters if it cancels an in-progress selection.
        return match selected_source {
            Some(_) => ClickOutcome::SetSource(None),
            None => ClickOutcome::Nothing,
        };
    };

    // A single-area move on the clicked area submits immediately.
    if let Some((idx, _)) = moves
        .iter()
        .find(|(_, m)| single_target_area(m) == Some(clicked))
    {
        return ClickOutcome::Submit(*idx);
    }

    // A valid target from the chosen source submits.
    if let Some(src) = selected_source
        && let Some((idx, _)) = moves
            .iter()
            .find(|(_, m)| move_endpoints(m) == Some((src, clicked)))
    {
        return ClickOutcome::Submit(*idx);
    }

    // Otherwise (re)pick a source if the clicked area is one, else clear.
    let clicked_is_source = moves
        .iter()
        .any(|(_, m)| move_endpoints(m).map(|(s, _)| s) == Some(clicked));
    let new_source = clicked_is_source.then_some(clicked);
    if new_source == selected_source {
        ClickOutcome::Nothing
    } else {
        ClickOutcome::SetSource(new_source)
    }
}

/// Translate a left click on the map into a move submission. Single-area
/// moves submit on one click; movement moves use source→target two-click.
/// The side-panel buttons remain available for non-spatial moves.
fn handle_map_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::civilization::GameCamera>>,
    map_state: Res<NetMapState>,
    mut net: ResMut<NetGame>,
    mut submit: MessageWriter<SubmitNetMove>,
) {
    if !mouse.just_pressed(MouseButton::Left) || net.moves.is_empty() {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    // Clicks over the side panel belong to its buttons, not the map.
    if cursor.x >= window.width() - PANEL_WIDTH {
        return;
    }
    let Ok((camera, cam_tf)) = camera_q.single() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) else {
        return;
    };

    // Nearest area within the click radius (None = clicked empty sea).
    let mut clicked: Option<AreaId> = None;
    let mut best = CLICK_RADIUS;
    for (id, pos) in &map_state.area_positions {
        let d = world.distance(*pos);
        if d <= best {
            best = d;
            clicked = Some(*id);
        }
    }

    match resolve_map_click(&net.moves, net.selected_source, clicked) {
        ClickOutcome::Submit(idx) => {
            submit.write(SubmitNetMove(idx));
            net.selected_source = None;
        }
        ClickOutcome::SetSource(source) => {
            net.selected_source = source;
            net.touch();
        }
        ClickOutcome::Nothing => {}
    }
}

/// Spawn dot markers on clickable areas: green = submittable now, yellow =
/// the selected movement source. Rebuilt whenever the net state changes.
fn refresh_map_highlights(
    mut commands: Commands,
    net: Res<NetGame>,
    map_state: Res<NetMapState>,
    textures: Res<crate::loading::TextureAssets>,
    existing: Query<Entity, With<NetMoveHighlight>>,
) {
    if !net.is_changed() {
        return;
    }
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }
    if net.moves.is_empty() {
        return;
    }

    let green = Color::srgba(0.2, 1.0, 0.35, 0.55);
    let yellow = Color::srgba(1.0, 0.9, 0.2, 0.75);
    let spawn = |commands: &mut Commands, id: AreaId, color: Color| {
        if let Some(pos) = map_state.area_positions.get(&id) {
            commands.spawn((
                NetMoveHighlight,
                Sprite {
                    image: textures.dot.clone(),
                    color,
                    custom_size: Some(Vec2::splat(34.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 4.0),
            ));
        }
    };

    // Single-area moves: highlight every actionable area.
    let single: Vec<AreaId> = net
        .moves
        .iter()
        .filter_map(|(_, m)| single_target_area(m))
        .collect();
    if !single.is_empty() {
        for area in &single {
            spawn(&mut commands, *area, green);
        }
        debug!("Map highlights: {} single-area targets", single.len());
        return;
    }

    // Movement: sources to pick from, or — once a source is chosen — its
    // reachable targets.
    if let Some(src) = net.selected_source {
        spawn(&mut commands, src, yellow);
        for (_, m) in &net.moves {
            if let Some((s, t)) = move_endpoints(m)
                && s == src
            {
                spawn(&mut commands, t, green);
            }
        }
    } else {
        let mut seen = std::collections::HashSet::new();
        for (_, m) in &net.moves {
            if let Some((s, _)) = move_endpoints(m)
                && seen.insert(s)
            {
                spawn(&mut commands, s, green);
            }
        }
    }
}

fn join_when_connected(
    connected: Query<&mut MessageSender<JoinGame>, Added<Connected>>,
    settings: Res<NetworkSettings>,
    token_auth: Res<UsedTokenAuth>,
    mut net: ResMut<NetGame>,
) {
    for mut sender in connected {
        net.connected = true;
        net.touch();
        // Token sessions were seated by the HTTP join (the server matched
        // our netcode client id) — sending JoinGame too would race it.
        if token_auth.0 {
            info!("Connected — seat was claimed via the join API");
            continue;
        }
        info!("Connected — joining as {}", settings.player_name);
        sender.send::<ControlChannel>(JoinGame {
            player_name: settings.player_name.clone(),
            // Manual auth (this path only) has no HTTP round trip to carry
            // one; the server accepts a plain name match here, same as
            // before this change.
            reconnect_token: None,
        });
    }
}

#[allow(clippy::type_complexity)]
fn receive_net_messages(
    mut accepted: Query<&mut MessageReceiver<JoinAccepted>>,
    mut lobby: Query<&mut MessageReceiver<LobbyState>>,
    mut phases: Query<&mut MessageReceiver<PhaseChanged>>,
    mut moves: Query<&mut MessageReceiver<YourMoves>>,
    mut rejected: Query<&mut MessageReceiver<MoveRejected>>,
    mut board: Query<&mut MessageReceiver<GameStateView>>,
    mut hands: Query<&mut MessageReceiver<YourHand>>,
    mut net: ResMut<NetGame>,
) {
    for mut receiver in &mut accepted {
        for msg in receiver.receive() {
            net.seated_as = Some((msg.player_name, msg.faction));
            net.touch();
        }
    }
    for mut receiver in &mut lobby {
        for msg in receiver.receive() {
            net.lobby = Some(msg);
            net.touch();
        }
    }
    for mut receiver in &mut phases {
        for msg in receiver.receive() {
            net.phase = Some(msg.phase);
            // Stale moves (and any in-progress selection) don't survive a
            // phase change.
            net.moves.clear();
            net.selected_source = None;
            net.touch();
        }
    }
    for mut receiver in &mut moves {
        for msg in receiver.receive() {
            net.moves = msg.moves;
            net.selected_source = None;
            net.touch();
        }
    }
    for mut receiver in &mut rejected {
        for msg in receiver.receive() {
            net.last_error = Some(format!("move {} rejected: {}", msg.move_index, msg.reason));
            net.touch();
        }
    }
    for mut receiver in &mut board {
        for msg in receiver.receive() {
            net.board = Some(msg);
            net.touch();
        }
    }
    for mut receiver in &mut hands {
        for msg in receiver.receive() {
            net.hand = msg.cards;
            net.touch();
        }
    }
}

fn forward_submitted_moves(
    mut submitted: MessageReader<SubmitNetMove>,
    mut senders: Query<&mut MessageSender<SubmitMove>>,
    mut net: ResMut<NetGame>,
) {
    for SubmitNetMove(index) in submitted.read() {
        for mut sender in &mut senders {
            sender.send::<ControlChannel>(SubmitMove::index(*index));
        }
        // Optimistic: hide the move list until the server answers with
        // fresh moves or a rejection.
        net.moves.clear();
        net.last_error = None;
        net.touch();
    }
}

/// Crude but effective: tear the whole screen down and rebuild it whenever
/// anything changed. Fine at the rate a board game changes.
fn rebuild_online_ui(
    mut commands: Commands,
    mut net: ResMut<NetGame>,
    roots: Query<Entity, With<OnlineUiRoot>>,
    theme: Res<LavaTheme>,
) {
    if !net.dirty {
        return;
    }
    net.dirty = false;
    for root in roots.iter() {
        commands.entity(root).despawn();
    }

    // Right-side panel; the map view owns the rest of the screen.
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));
    ui.component::<OnlineUiRoot>()
        .absolute_position()
        .right(px(0.0))
        .top(px(0.0))
        .width(px(460.0))
        .height_percent(100.0)
        .display_flex()
        .flex_column()
        .align_items_start()
        .padding_all_px(14.0)
        .gap_px(8.0);

    // ── Header ──────────────────────────────────────────────────────────
    let title = match (&net.seated_as, net.connected) {
        (Some((name, faction)), _) => format!("{name} — {faction}"),
        (None, true) => "Joining…".into(),
        (None, false) => "Connecting…".into(),
    };
    ui.add_text_child(title, Some(TextStyle::size(28.0)));
    if let Some(phase) = &net.phase {
        ui.add_text_child(format!("Phase: {phase:?}"), Some(TextStyle::size(20.0)));
    }
    if let Some(error) = &net.last_error {
        ui.add_text_child(error.clone(), Some(TextStyle::size(16.0)));
    }

    // ── Lobby (before the game starts) ──────────────────────────────────
    if net.phase.is_none()
        && let Some(lobby) = &net.lobby
    {
        ui.add_text_child(
            format!("Lobby — {} seats", lobby.seats_total),
            Some(TextStyle::size(22.0)),
        );
        for player in &lobby.players {
            let status = if player.connected { "✓" } else { "…" };
            ui.add_text_child(
                format!("{status} {} — {}", player.faction, player.name),
                Some(TextStyle::size(18.0)),
            );
        }
    }

    // ── Your moves ───────────────────────────────────────────────────────
    if !net.moves.is_empty() {
        ui.add_text_child("Your moves:", Some(TextStyle::size(20.0)));
        // Spatial hint: the map is the primary input; buttons are a fallback.
        let map_clickable = net
            .moves
            .iter()
            .any(|(_, m)| single_target_area(m).is_some() || move_endpoints(m).is_some());
        if map_clickable {
            let hint = match net.selected_source {
                Some(_) => "Click a green target on the map (or empty sea to cancel).",
                None => "Click a highlighted area on the map, or a button below.",
            };
            ui.add_text_child(hint, Some(TextStyle::size(14.0)));
        }
        for (index, game_move) in net.moves.iter().take(12) {
            let index = *index;
            let label = describe_net_move(game_move);
            ui.add_button_observe(
                label,
                |btn| {
                    btn.size(px(520.0), px(36.0));
                },
                move |_: On<bevy::ui_widgets::Activate>,
                      mut writer: MessageWriter<SubmitNetMove>| {
                    writer.write(SubmitNetMove(index));
                },
            );
        }
        if net.moves.len() > 12 {
            ui.add_text_child(
                format!("…and {} more", net.moves.len() - 12),
                Some(TextStyle::size(14.0)),
            );
        }
    }

    // ── Hand ─────────────────────────────────────────────────────────────
    if !net.hand.is_empty() {
        let cards: Vec<String> = net
            .hand
            .iter()
            .map(|(card, count)| format!("{card} ×{count}"))
            .collect();
        ui.add_text_child(
            format!("Hand: {}", cards.join(", ")),
            Some(TextStyle::size(16.0)),
        );
    }

    // ── Board summary ────────────────────────────────────────────────────
    if let Some(board) = &net.board {
        for player in &board.players {
            ui.add_text_child(
                format!(
                    "{} ({}) — stock {}, {} trade cards, {} civ cards",
                    player.name,
                    player.faction,
                    player.tokens_in_stock,
                    player.trade_card_count,
                    player.civ_cards.len()
                ),
                Some(TextStyle::size(15.0)),
            );
        }
    }

    // ── Leave ────────────────────────────────────────────────────────────
    ui.add_button_observe(
        "Leave game",
        |btn| {
            btn.size(px(200.0), px(40.0));
        },
        |_: On<bevy::ui_widgets::Activate>, mut next_state: ResMut<NextState<GameState>>| {
            next_state.set(GameState::Menu);
        },
    );

    ui.build();
}

fn describe_net_move(game_move: &NetGameMove) -> String {
    match game_move {
        NetGameMove::PopulationExpansion { area, max_tokens } => {
            format!("Expand {area} (up to {max_tokens})")
        }
        NetGameMove::Movement {
            source,
            target,
            max_tokens,
        } => format!("Move {source} → {target} (up to {max_tokens})"),
        NetGameMove::ShipFerry {
            source,
            target,
            max_tokens,
        } => format!("Ferry {source} → {target} (up to {max_tokens})"),
        NetGameMove::AttackArea {
            source,
            target,
            max_tokens: _,
        } => format!("Attack {target} from {source}"),
        NetGameMove::AttackCity {
            source,
            target,
            max_tokens: _,
        } => format!("Attack city in {target} from {source}"),
        NetGameMove::EndMovement => "End movement".into(),
        NetGameMove::BuildCity { area } => format!("Build city in {area}"),
        NetGameMove::EndCityConstruction => "End city construction".into(),
        NetGameMove::EliminateCity { area, .. } => format!("Eliminate city in {area}"),
        NetGameMove::Trade(trade) => match trade {
            NetTradeMove::ProposeTrade { to, .. } => format!("Propose trade with {to}"),
            NetTradeMove::AcceptOrDeclineTrade { .. } => "Respond to trade offer".into(),
            NetTradeMove::AutoDeclineTrade { .. } => "Decline trade offer".into(),
            NetTradeMove::StopTrading => "Stop trading".into(),
            NetTradeMove::SettleTrade { .. } => "Settle trade".into(),
        },
        NetGameMove::AcquireCivCards { cards } => {
            let names: Vec<String> = cards.iter().map(std::string::ToString::to_string).collect();
            format!("Buy {}", names.join(" + "))
        }
        NetGameMove::DoneAcquiringCivCards => "Done buying civilization cards".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pop_exp(area: i32) -> NetGameMove {
        NetGameMove::PopulationExpansion {
            area: AreaId(area),
            max_tokens: 1,
        }
    }
    fn movement(source: i32, target: i32) -> NetGameMove {
        NetGameMove::Movement {
            source: AreaId(source),
            target: AreaId(target),
            max_tokens: 3,
        }
    }

    #[test]
    fn single_area_move_submits_on_click() {
        let moves = vec![(4, pop_exp(10)), (7, pop_exp(20))];
        assert_eq!(
            resolve_map_click(&moves, None, Some(AreaId(20))),
            ClickOutcome::Submit(7)
        );
    }

    #[test]
    fn clicking_irrelevant_area_does_nothing() {
        let moves = vec![(4, pop_exp(10))];
        // Area 99 is neither a single-target nor a movement source.
        assert_eq!(
            resolve_map_click(&moves, None, Some(AreaId(99))),
            ClickOutcome::Nothing
        );
    }

    #[test]
    fn first_movement_click_selects_source() {
        let moves = vec![(1, movement(5, 6)), (2, movement(5, 7))];
        assert_eq!(
            resolve_map_click(&moves, None, Some(AreaId(5))),
            ClickOutcome::SetSource(Some(AreaId(5)))
        );
    }

    #[test]
    fn second_movement_click_submits_matching_target() {
        let moves = vec![(1, movement(5, 6)), (2, movement(5, 7))];
        assert_eq!(
            resolve_map_click(&moves, Some(AreaId(5)), Some(AreaId(7))),
            ClickOutcome::Submit(2)
        );
    }

    #[test]
    fn clicking_another_source_switches_selection() {
        let moves = vec![(1, movement(5, 6)), (2, movement(8, 9))];
        assert_eq!(
            resolve_map_click(&moves, Some(AreaId(5)), Some(AreaId(8))),
            ClickOutcome::SetSource(Some(AreaId(8)))
        );
    }

    #[test]
    fn clicking_invalid_target_clears_selection() {
        let moves = vec![(1, movement(5, 6))];
        // Area 42 is not a target of source 5 and not itself a source.
        assert_eq!(
            resolve_map_click(&moves, Some(AreaId(5)), Some(AreaId(42))),
            ClickOutcome::SetSource(None)
        );
    }

    #[test]
    fn clicking_empty_sea_cancels_source_but_is_otherwise_inert() {
        let moves = vec![(1, movement(5, 6))];
        assert_eq!(
            resolve_map_click(&moves, Some(AreaId(5)), None),
            ClickOutcome::SetSource(None)
        );
        assert_eq!(resolve_map_click(&moves, None, None), ClickOutcome::Nothing);
    }

    #[test]
    fn single_area_takes_priority_over_movement_source() {
        // A build-city target that is also (hypothetically) a movement source
        // resolves as the direct single-area submit.
        let moves = vec![
            (3, NetGameMove::BuildCity { area: AreaId(12) }),
            (4, movement(12, 13)),
        ];
        assert_eq!(
            resolve_map_click(&moves, None, Some(AreaId(12))),
            ClickOutcome::Submit(3)
        );
    }
}
