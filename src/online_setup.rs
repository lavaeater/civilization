//! "Play Online" → Create / Join screen (docs/online-games-plan.md, step 4).
//!
//! - **Create:** pick total players and human seats → `POST /api/games` on
//!   the lobby (`civ_lobby`) → you get a code and an invite link, and join.
//! - **Join:** type a code (or open a `?game=<code>` link, which prefills
//!   it) → `GET /api/games/<code>` checks it exists → join.
//!
//! Either way the game's API lives at `<lobby>/g/<code>`; the join response
//! hands back the game's own WebSocket URL, so `network_client` needs no
//! other change. Against a server that is *not* a lobby (a bare
//! `adv_civ_server` hosting one game) both buttons fall back to joining
//! that one game directly, as "Play Online" used to.

use crate::GameState;
use crate::network_client::NetworkSettings;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use lava_ui_builder::{LavaTheme, TextStyle, UIBuilder};
use std::sync::Mutex;
use std::sync::mpsc;

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 9;

pub struct OnlineSetupPlugin;

impl Plugin for OnlineSetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SetupForm>()
            .add_systems(OnEnter(GameState::OnlineSetup), spawn_setup_screen)
            .add_systems(OnExit(GameState::OnlineSetup), despawn_setup_screen)
            .add_systems(
                Update,
                (auto_setup, poll_lobby_reply, refresh_labels)
                    .chain()
                    .run_if(in_state(GameState::OnlineSetup)),
            )
            .add_systems(Update, open_invite_link.run_if(in_state(GameState::Menu)));
    }
}

#[derive(Resource)]
pub struct SetupForm {
    pub total_players: usize,
    pub human_seats: usize,
    /// A request is in flight; buttons do nothing until it answers.
    busy: bool,
    message: Option<String>,
}

impl Default for SetupForm {
    fn default() -> Self {
        SetupForm {
            total_players: 4,
            human_seats: 2,
            busy: false,
            message: None,
        }
    }
}

impl SetupForm {
    pub fn change_players(&mut self, delta: isize) {
        self.total_players = self
            .total_players
            .saturating_add_signed(delta)
            .clamp(MIN_PLAYERS, MAX_PLAYERS);
        self.human_seats = self.human_seats.min(self.total_players);
    }

    pub fn change_humans(&mut self, delta: isize) {
        self.human_seats = self
            .human_seats
            .saturating_add_signed(delta)
            .clamp(1, self.total_players);
    }
}

#[derive(Component, Default)]
struct SetupRoot;
#[derive(Component)]
struct NameField;
#[derive(Component)]
struct CodeField;
#[derive(Component)]
struct PlayersLabel;
#[derive(Component)]
struct HumansLabel;
#[derive(Component)]
struct MessageLabel;

/// What the lobby said, boiled down.
#[derive(Debug, PartialEq, Eq)]
pub enum LobbyReply {
    Game {
        code: String,
        join_url: Option<String>,
    },
    NoSuchGame,
    /// The server answered but isn't a lobby: it hosts a single game.
    NotALobby,
    Error(String),
}

/// `(player name, reply)`, sent back from the request thread / future.
#[derive(Resource)]
struct LobbyFetch(Mutex<mpsc::Receiver<(String, LobbyReply)>>);

/// When the page was opened from an invite link (`?game=<code>`), skip the
/// main menu straight to this screen, once.
fn open_invite_link(
    settings: Res<NetworkSettings>,
    mut next: ResMut<NextState<GameState>>,
    mut done: Local<bool>,
) {
    if !*done {
        *done = true;
        if settings.game_code.is_some() {
            next.set(GameState::OnlineSetup);
        }
    }
}

const FIELD_BG: Color = Color::srgb(0.08, 0.1, 0.12);
const FIELD_BORDER: Color = Color::srgb(0.45, 0.45, 0.3);

fn text_field<M: Component>(ui: &mut UIBuilder, initial: &str, max: usize, marker: M) {
    ui.with_child(|field| {
        field
            .width_px(320.0)
            .padding_all_px(8.0)
            .bg_color(FIELD_BG)
            .border_all_px(2.0, FIELD_BORDER)
            .insert_bundle((
                EditableText {
                    max_characters: Some(max),
                    ..EditableText::new(initial)
                },
                TextFont::default().with_font_size(22.0),
                TextColor(Color::WHITE),
                TextCursorStyle::default(),
                TabIndex(0),
                marker,
            ));
    });
}

fn stepper<M: Component>(
    ui: &mut UIBuilder,
    caption: &str,
    marker: M,
    on_change: fn(&mut SetupForm, isize),
) {
    ui.add_row(|row| {
        row.align_items_center().gap_px(12.0);
        row.with_child(|label| {
            label.width_px(180.0);
            label.with_text(caption, Some(TextStyle::size(22.0)));
        });
        row.add_button_observe(
            "-",
            |btn| {
                btn.size(px(44.0), px(44.0));
            },
            move |_: On<bevy::ui_widgets::Activate>, mut form: ResMut<SetupForm>| {
                on_change(&mut form, -1);
            },
        );
        row.with_child(|value| {
            value.width_px(40.0);
            value
                .with_text("", Some(TextStyle::size(26.0)))
                .insert(marker);
        });
        row.add_button_observe(
            "+",
            |btn| {
                btn.size(px(44.0), px(44.0));
            },
            move |_: On<bevy::ui_widgets::Activate>, mut form: ResMut<SetupForm>| {
                on_change(&mut form, 1);
            },
        );
    });
}

fn spawn_setup_screen(
    commands: Commands,
    theme: Res<LavaTheme>,
    settings: Res<NetworkSettings>,
    mut form: ResMut<SetupForm>,
) {
    form.busy = false;
    form.message = None;
    form.set_changed();

    let mut ui = UIBuilder::new(commands, Some(theme.clone()));
    ui.component::<SetupRoot>()
        .size_percent(100.0, 100.0)
        .display_flex()
        .flex_column()
        .align_items_center()
        .justify_center()
        .gap_px(14.0)
        .bg_color(Color::srgba(0.05, 0.1, 0.15, 0.97));

    ui.add_text_child("Play Online", Some(TextStyle::size(44.0)));

    ui.add_text_child("Your name", Some(TextStyle::size(20.0)));
    text_field(&mut ui, &settings.player_name, 24, NameField);

    ui.add_row(|columns| {
        columns.gap_px(60.0).margin_top(px(20.0));

        // ── Create ───────────────────────────────────────────────────────
        columns.add_column(|col| {
            col.gap_px(10.0).align_items_start();
            col.add_text_child("Create a new game", Some(TextStyle::size(28.0)));
            stepper(col, "Players", PlayersLabel, SetupForm::change_players);
            stepper(col, "of which human", HumansLabel, SetupForm::change_humans);
            col.add_text_child(
                "The rest are played by the computer.",
                Some(TextStyle::size(16.0)),
            );
            col.add_button_observe(
                "Create game",
                |btn| {
                    btn.size(px(320.0), px(56.0));
                },
                create_clicked,
            );
        });

        // ── Join ─────────────────────────────────────────────────────────
        columns.add_column(|col| {
            col.gap_px(10.0).align_items_start();
            col.add_text_child("Join a game", Some(TextStyle::size(28.0)));
            col.add_text_child("Game code", Some(TextStyle::size(20.0)));
            text_field(
                col,
                settings.game_code.as_deref().unwrap_or(""),
                40,
                CodeField,
            );
            col.add_button_observe(
                "Join game",
                |btn| {
                    btn.size(px(320.0), px(56.0));
                },
                join_clicked,
            );
        });
    });

    ui.with_child(|msg| {
        msg.margin_top(px(16.0));
        msg.with_text("", Some(TextStyle::size(20.0)))
            .insert(MessageLabel);
    });

    ui.add_button_observe(
        "Back",
        |btn| {
            btn.size(px(200.0), px(48.0));
        },
        |_: On<bevy::ui_widgets::Activate>, mut next: ResMut<NextState<GameState>>| {
            next.set(GameState::Menu);
        },
    );
    ui.build();
}

fn despawn_setup_screen(mut commands: Commands, roots: Query<Entity, With<SetupRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<LobbyFetch>();
}

#[allow(clippy::type_complexity)]
fn refresh_labels(
    form: Res<SetupForm>,
    mut labels: ParamSet<(
        Query<&mut Text, With<PlayersLabel>>,
        Query<&mut Text, With<HumansLabel>>,
        Query<&mut Text, With<MessageLabel>>,
    )>,
) {
    if !form.is_changed() {
        return;
    }
    for mut text in &mut labels.p0() {
        text.0 = form.total_players.to_string();
    }
    for mut text in &mut labels.p1() {
        text.0 = form.human_seats.to_string();
    }
    for mut text in &mut labels.p2() {
        text.0 = form.message.clone().unwrap_or_default();
    }
}

fn field_value<M: Component>(fields: &Query<&EditableText, With<M>>) -> String {
    fields
        .iter()
        .next()
        .map_or_else(String::new, |f| f.value().to_string().trim().to_string())
}

fn read_name(names: &Query<&EditableText, With<NameField>>, form: &mut SetupForm) -> Option<String> {
    let name = field_value(names);
    if name.is_empty() {
        form.message = Some("Type your name first.".into());
        None
    } else {
        Some(name)
    }
}

fn create_clicked(
    _: On<bevy::ui_widgets::Activate>,
    mut commands: Commands,
    mut form: ResMut<SetupForm>,
    settings: Res<NetworkSettings>,
    names: Query<&EditableText, With<NameField>>,
) {
    start_create(&mut commands, &mut form, &settings, &names);
}

fn start_create(
    commands: &mut Commands,
    form: &mut SetupForm,
    settings: &NetworkSettings,
    names: &Query<&EditableText, With<NameField>>,
) {
    if form.busy {
        return;
    }
    let Some(name) = read_name(names, form) else {
        return;
    };
    let Some(lobby) = settings.lobby_url.clone() else {
        form.message = Some("No online server configured (LOBBY_URL).".into());
        return;
    };
    form.busy = true;
    form.message = Some("Creating game…".into());
    let body = serde_json::json!({
        "total_players": form.total_players,
        "human_seats": form.human_seats,
    })
    .to_string();
    let (tx, rx) = mpsc::channel();
    commands.insert_resource(LobbyFetch(Mutex::new(rx)));
    http::request("POST", format!("{lobby}/api/games"), Some(body), move |result| {
        let _ = tx.send((name, interpret(result)));
    });
}

fn join_clicked(
    _: On<bevy::ui_widgets::Activate>,
    mut commands: Commands,
    mut form: ResMut<SetupForm>,
    settings: Res<NetworkSettings>,
    names: Query<&EditableText, With<NameField>>,
    codes: Query<&EditableText, With<CodeField>>,
) {
    start_join_lookup(&mut commands, &mut form, &settings, &names, &codes);
}

fn start_join_lookup(
    commands: &mut Commands,
    form: &mut SetupForm,
    settings: &NetworkSettings,
    names: &Query<&EditableText, With<NameField>>,
    codes: &Query<&EditableText, With<CodeField>>,
) {
    if form.busy {
        return;
    }
    let Some(name) = read_name(names, form) else {
        return;
    };
    let Some(code) = normalize_code(&field_value(codes)) else {
        form.message = Some("Type the game code you were sent.".into());
        return;
    };
    let Some(lobby) = settings.lobby_url.clone() else {
        form.message = Some("No online server configured (LOBBY_URL).".into());
        return;
    };
    form.busy = true;
    form.message = Some(format!("Looking for {code}…"));
    let (tx, rx) = mpsc::channel();
    commands.insert_resource(LobbyFetch(Mutex::new(rx)));
    http::request("GET", format!("{lobby}/api/games/{code}"), None, move |result| {
        let _ = tx.send((name, interpret(result)));
    });
}

/// Dev convenience, like `AUTO_ONLINE`: `AUTO_ONLINE_SETUP=create` or
/// `=join` presses that button as soon as the screen is up (the fields are
/// prefilled from `PLAYER_NAME` / `GAME_CODE`). Lets the whole create/join
/// flow run without a mouse.
fn auto_setup(
    mut commands: Commands,
    mut form: ResMut<SetupForm>,
    settings: Res<NetworkSettings>,
    names: Query<&EditableText, With<NameField>>,
    codes: Query<&EditableText, With<CodeField>>,
    mut done: Local<bool>,
) {
    if *done || names.is_empty() {
        return;
    }
    *done = true;
    match std::env::var("AUTO_ONLINE_SETUP").as_deref() {
        Ok("create") => start_create(&mut commands, &mut form, &settings, &names),
        Ok("join") => start_join_lookup(&mut commands, &mut form, &settings, &names, &codes),
        _ => {}
    }
}

fn poll_lobby_reply(
    mut commands: Commands,
    fetch: Option<Res<LobbyFetch>>,
    mut form: ResMut<SetupForm>,
    mut settings: ResMut<NetworkSettings>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Some(fetch) = fetch else {
        return;
    };
    let reply = match fetch.0.lock() {
        Ok(rx) => match rx.try_recv() {
            Ok(reply) => reply,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                (String::new(), LobbyReply::Error("request abandoned".into()))
            }
        },
        Err(_) => return,
    };
    commands.remove_resource::<LobbyFetch>();
    form.busy = false;

    let (name, reply) = reply;
    let Some(lobby) = settings.lobby_url.clone() else {
        return;
    };
    match reply {
        LobbyReply::Game { code, join_url } => {
            info!("Joining online game {code}");
            settings.invite_link = Some(join_url.unwrap_or_else(|| invite_link(&lobby, &code)));
            settings.api_url = Some(format!("{lobby}/g/{code}"));
            settings.game_code = Some(code);
            settings.player_name = name;
            next.set(GameState::Online);
        }
        LobbyReply::NotALobby => {
            info!("{lobby} hosts a single game; joining it directly");
            settings.api_url = Some(lobby);
            settings.game_code = None;
            settings.invite_link = None;
            settings.player_name = name;
            next.set(GameState::Online);
        }
        LobbyReply::NoSuchGame => {
            form.message = Some("No game with that code. Check the spelling?".into());
        }
        LobbyReply::Error(e) => {
            form.message = Some(e);
        }
    }
}

/// The link to send friends. In the browser it's this page with `?game=`;
/// natively, the lobby's own address (where the web client is served).
fn invite_link(lobby: &str, code: &str) -> String {
    #[cfg(target_family = "wasm")]
    {
        let _ = lobby;
        let location = web_sys::window().map(|w| w.location());
        let origin = location
            .as_ref()
            .and_then(|l| l.origin().ok())
            .unwrap_or_default();
        let path = location
            .as_ref()
            .and_then(|l| l.pathname().ok())
            .unwrap_or_else(|| "/".into());
        format!("{origin}{path}?game={code}")
    }
    #[cfg(not(target_family = "wasm"))]
    format!("{lobby}/?game={code}")
}

/// Same rules as the lobby's: lowercase letters, digits, hyphens.
pub fn normalize_code(input: &str) -> Option<String> {
    let code = input.trim().to_ascii_lowercase().replace(' ', "-");
    let valid = !code.is_empty()
        && code.len() <= 40
        && code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    valid.then_some(code)
}

/// `(status, body)` or a transport error → what it means for us.
pub fn interpret(result: Result<(u16, String), String>) -> LobbyReply {
    let (status, body) = match result {
        Ok(ok) => ok,
        Err(e) => return LobbyReply::Error(format!("Could not reach the server: {e}")),
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else {
        // e.g. a bare game server answering an unknown GET with its SPA page.
        return LobbyReply::NotALobby;
    };
    if let Some(code) = value["code"].as_str() {
        return LobbyReply::Game {
            code: code.to_string(),
            join_url: value["join_url"].as_str().map(str::to_string),
        };
    }
    match value["error"].as_str() {
        Some("no such game") => LobbyReply::NoSuchGame,
        Some(_) if status == 404 || status == 405 => LobbyReply::NotALobby,
        Some(error) => LobbyReply::Error(error.to_string()),
        None if status == 404 => LobbyReply::NotALobby,
        None => LobbyReply::Error(format!("Unexpected answer from the server ({status})")),
    }
}

/// One JSON request, answered through a callback from another thread
/// (native) or a future (wasm). 4xx/5xx come back as `Ok((status, body))`.
mod http {
    type Reply = Result<(u16, String), String>;

    #[cfg(not(target_family = "wasm"))]
    pub fn request(
        method: &'static str,
        url: String,
        body: Option<String>,
        done: impl FnOnce(Reply) + Send + 'static,
    ) {
        std::thread::spawn(move || {
            let request = ureq::request(method, &url)
                .set("Content-Type", "application/json")
                .timeout(std::time::Duration::from_secs(40));
            let response = match body {
                Some(body) => request.send_string(&body),
                None => request.call(),
            };
            let reply = match response {
                Ok(response) => {
                    let status = response.status();
                    response
                        .into_string()
                        .map(|body| (status, body))
                        .map_err(|e| e.to_string())
                }
                Err(ureq::Error::Status(status, response)) => {
                    Ok((status, response.into_string().unwrap_or_default()))
                }
                Err(e) => Err(e.to_string()),
            };
            done(reply);
        });
    }

    #[cfg(target_family = "wasm")]
    pub fn request(
        method: &'static str,
        url: String,
        body: Option<String>,
        done: impl FnOnce(Reply) + 'static,
    ) {
        use gloo_net::http::Request;
        wasm_bindgen_futures::spawn_local(async move {
            let reply = async {
                let builder = match method {
                    "POST" => Request::post(&url),
                    _ => Request::get(&url),
                }
                .header("Content-Type", "application/json");
                let request = match body {
                    Some(body) => builder.body(body).map_err(|e| e.to_string())?,
                    None => builder.build().map_err(|e| e.to_string())?,
                };
                let response = request.send().await.map_err(|e| e.to_string())?;
                let status = response.status();
                let text = response.text().await.map_err(|e| e.to_string())?;
                Ok((status, text))
            }
            .await;
            done(reply);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steppers_stay_in_range_and_humans_never_exceed_players() {
        let mut form = SetupForm::default();
        for _ in 0..20 {
            form.change_players(1);
        }
        assert_eq!(form.total_players, MAX_PLAYERS);
        for _ in 0..20 {
            form.change_humans(1);
        }
        assert_eq!(form.human_seats, MAX_PLAYERS);
        for _ in 0..20 {
            form.change_players(-1);
        }
        assert_eq!(form.total_players, MIN_PLAYERS);
        assert_eq!(form.human_seats, MIN_PLAYERS);
        for _ in 0..5 {
            form.change_humans(-1);
        }
        assert_eq!(form.human_seats, 1);
    }

    #[test]
    fn lobby_answers_are_interpreted() {
        let ok = |status, body: &str| interpret(Ok((status, body.to_string())));
        assert_eq!(
            ok(200, r#"{"code":"brisk-otter-42","join_url":null}"#),
            LobbyReply::Game {
                code: "brisk-otter-42".into(),
                join_url: None
            }
        );
        assert_eq!(ok(404, r#"{"error":"no such game"}"#), LobbyReply::NoSuchGame);
        // A bare adv_civ_server: unknown POST → JSON 404, unknown GET → HTML.
        assert_eq!(ok(404, r#"{"error":"not found"}"#), LobbyReply::NotALobby);
        assert_eq!(ok(200, "<!doctype html>"), LobbyReply::NotALobby);
        assert!(matches!(
            ok(429, r#"{"error":"too many games created from here recently"}"#),
            LobbyReply::Error(_)
        ));
        assert!(matches!(interpret(Err("refused".into())), LobbyReply::Error(_)));
    }

    #[test]
    fn codes_are_normalized() {
        assert_eq!(normalize_code(" Brisk-Otter-42 ").as_deref(), Some("brisk-otter-42"));
        assert_eq!(normalize_code("brisk otter 42").as_deref(), Some("brisk-otter-42"));
        assert_eq!(normalize_code("../x"), None);
        assert_eq!(normalize_code(""), None);
    }
}
