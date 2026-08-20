//! Minimal interactive CLI client for the multiplayer spike.
//!
//! Connects to the local spike server over plain WebSocket, claims a seat,
//! prints the board state and your available moves, and submits the move
//! index you type. Usage:
//!
//! ```sh
//! cargo run -p adv_civ_server --bin spike_client -- Tommie
//! # then type e.g. `0` (move index) or `0 2` (move index + token count)
//! ```

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

use adv_civ_protocol::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

const TICK_HZ: f64 = 32.0;
const SERVER_PORT: u16 = 5111;
const SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SERVER_PORT);

#[derive(Resource)]
struct PlayerName(String);

#[derive(Resource)]
struct StdinLines(Mutex<Receiver<String>>);

/// `--auto`: play unattended — prefer the phase-ending move when one is
/// offered, otherwise take the first. Exercises the protocol end to end.
#[derive(Resource)]
struct AutoPlay(bool);

/// Base64 ConnectToken from the HTTP join endpoint (`--token <b64>`).
/// Without it, falls back to manual zero-key dev auth + a JoinGame message.
#[derive(Resource, Default)]
struct JoinToken(Option<String>);

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Anonymous".into());
    let auto = std::env::args().any(|a| a == "--auto");
    let token = {
        let args: Vec<String> = std::env::args().collect();
        args.iter()
            .position(|a| a == "--token")
            .and_then(|i| args.get(i + 1).cloned())
    };

    let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            match stdin.read_line(&mut line) {
                // 0 bytes read = EOF; without this the loop spins forever
                // flooding the channel with empty strings.
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && tx.send(trimmed.to_string()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    )
    .add_plugins(LogPlugin {
        filter: "warn,spike_client=info".into(),
        ..default()
    });

    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.add_plugins(ProtocolPlugin);

    app.insert_resource(PlayerName(name));
    app.insert_resource(StdinLines(Mutex::new(rx)));
    app.insert_resource(AutoPlay(auto));
    app.insert_resource(JoinToken(token));

    app.add_systems(Startup, connect);
    app.add_systems(
        Update,
        (join_when_connected, receive_messages, submit_typed_moves),
    );

    app.run();
}

fn connect(mut commands: Commands, token: Res<JoinToken>) -> Result {
    let auth = if let Some(token_b64) = &token.0 {
        // Real flow: short-lived ConnectToken from the HTTP join endpoint.
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(token_b64)
            .map_err(|e| format!("--token is not valid base64: {e}"))?;
        let token = lightyear::netcode::ConnectToken::try_from_bytes(&bytes)
            .map_err(|e| format!("--token is not a valid ConnectToken: {e}"))?;
        Authentication::Token(token)
    } else {
        // Dev-only manual auth: zero key, random client id. Works against a
        // server running with the default dev key.
        let client_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Authentication::Manual {
            server_addr: SERVER_ADDR,
            client_id,
            private_key: Key::default(),
            protocol_id: PROTOCOL_ID,
        }
    };
    let config = ClientConfig::builder().with_no_encryption();
    let client = commands
        .spawn((
            Client::default(),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            NetcodeClient::new(auth, NetcodeConfig::default())?,
            WebSocketClientIo::from_url(config, format!("ws://127.0.0.1:{SERVER_PORT}")),
        ))
        .id();
    commands.trigger(Connect { entity: client });
    println!("Connecting to {SERVER_ADDR} …");
    Ok(())
}

/// Claim a seat as soon as the connection handshake completes.
fn join_when_connected(
    connected: Query<&mut MessageSender<JoinGame>, Added<Connected>>,
    name: Res<PlayerName>,
) {
    for mut sender in connected {
        println!("Connected. Joining as {} …", name.0);
        sender.send::<ControlChannel>(JoinGame {
            player_name: name.0.clone(),
        });
    }
}

fn receive_messages(
    mut accepted: Query<&mut MessageReceiver<JoinAccepted>>,
    mut lobby: Query<&mut MessageReceiver<LobbyState>>,
    mut phases: Query<&mut MessageReceiver<PhaseChanged>>,
    mut moves: Query<&mut MessageReceiver<YourMoves>>,
    mut rejected: Query<&mut MessageReceiver<MoveRejected>>,
    mut state: Query<&mut MessageReceiver<GameStateView>>,
    mut hands: Query<&mut MessageReceiver<YourHand>>,
    mut submit: Query<&mut MessageSender<SubmitMove>>,
    auto_play: Res<AutoPlay>,
) {
    for mut receiver in hands.iter_mut() {
        for msg in receiver.receive() {
            let cards: Vec<String> = msg
                .cards
                .iter()
                .map(|(card, count)| format!("{card} ×{count}"))
                .collect();
            println!(
                "Your hand: {}",
                if cards.is_empty() {
                    "(empty)".into()
                } else {
                    cards.join(", ")
                }
            );
        }
    }
    for mut receiver in accepted.iter_mut() {
        for msg in receiver.receive() {
            println!("✓ Seated as {} ({})", msg.player_name, msg.faction);
        }
    }
    for mut receiver in lobby.iter_mut() {
        for msg in receiver.receive() {
            println!("Lobby ({} seats):", msg.seats_total);
            for p in &msg.players {
                let status = if p.connected { "joined" } else { "open" };
                println!("  {} — {} [{}]", p.faction, p.name, status);
            }
        }
    }
    for mut receiver in phases.iter_mut() {
        for msg in receiver.receive() {
            println!("— Phase: {:?} —", msg.phase);
        }
    }
    for mut receiver in state.iter_mut() {
        for msg in receiver.receive() {
            println!("Board:");
            for area in &msg.areas {
                let pops: Vec<String> = area
                    .population
                    .iter()
                    .map(|(f, n)| format!("{f}:{n}"))
                    .collect();
                println!(
                    "  [{}] {} (cap {}) {}",
                    area.area.0,
                    area.name,
                    area.max_population,
                    pops.join(" ")
                );
            }
            for player in &msg.players {
                println!(
                    "  {} ({}) — {} tokens in stock",
                    player.name, player.faction, player.tokens_in_stock
                );
            }
        }
    }
    for mut receiver in moves.iter_mut() {
        for msg in receiver.receive() {
            if msg.moves.is_empty() {
                println!("No moves available — waiting.");
                continue;
            }
            println!("Your moves (type an index, optionally `<index> <tokens>`):");
            for (index, game_move) in &msg.moves {
                println!("  {index}: {}", describe_move(game_move));
            }
            if auto_play.0 {
                // Build cities when possible (cities draw trade cards →
                // exercises the hidden-info path), otherwise wrap the phase
                // up, otherwise take whatever is first.
                let pick = msg
                    .moves
                    .iter()
                    .find(|(_, m)| matches!(m, NetGameMove::BuildCity { .. }))
                    .or_else(|| msg.moves.iter().find(|(_, m)| is_phase_ending(m)))
                    .or_else(|| msg.moves.first())
                    .map(|(i, _)| *i);
                if let Some(index) = pick {
                    println!("⚙ auto-picking move {index}");
                    for mut sender in submit.iter_mut() {
                        sender.send::<ControlChannel>(SubmitMove::index(index));
                    }
                }
            }
        }
    }
    for mut receiver in rejected.iter_mut() {
        for msg in receiver.receive() {
            println!("✗ Move {} rejected: {}", msg.move_index, msg.reason);
        }
    }
}

/// Moves that finish the player's involvement in the current phase.
fn is_phase_ending(game_move: &NetGameMove) -> bool {
    matches!(
        game_move,
        NetGameMove::EndMovement
            | NetGameMove::EndCityConstruction
            | NetGameMove::DoneAcquiringCivCards
            | NetGameMove::Trade(NetTradeMove::StopTrading)
    )
}

fn describe_move(game_move: &NetGameMove) -> String {
    match game_move {
        NetGameMove::PopulationExpansion { area, max_tokens } => {
            format!("expand {area} with up to {max_tokens} token(s)")
        }
        NetGameMove::Movement {
            source,
            target,
            max_tokens,
        } => format!("move up to {max_tokens} token(s) {source} → {target}"),
        NetGameMove::ShipFerry {
            source,
            target,
            max_tokens,
        } => format!("ferry up to {max_tokens} token(s) {source} → {target}"),
        NetGameMove::AttackArea {
            source,
            target,
            max_tokens,
        } => format!("attack {target} from {source} (up to {max_tokens})"),
        NetGameMove::AttackCity {
            source,
            target,
            max_tokens,
        } => format!("attack city in {target} from {source} (up to {max_tokens})"),
        NetGameMove::EndMovement => "end movement".into(),
        NetGameMove::BuildCity { area } => format!("build city in {area}"),
        NetGameMove::EndCityConstruction => "end city construction".into(),
        NetGameMove::EliminateCity {
            area,
            tokens_gained,
            tokens_needed,
        } => format!("eliminate city in {area} (gain {tokens_gained}, need {tokens_needed})"),
        NetGameMove::Trade(trade) => match trade {
            NetTradeMove::ProposeTrade { to, matching_cards } => {
                format!("propose trade with {to} (matching: {matching_cards:?})")
            }
            NetTradeMove::AcceptOrDeclineTrade { .. } => "accept/decline trade offer".into(),
            NetTradeMove::AutoDeclineTrade { .. } => "auto-decline trade offer".into(),
            NetTradeMove::StopTrading => "stop trading".into(),
            NetTradeMove::SettleTrade { .. } => "settle trade".into(),
        },
        NetGameMove::AcquireCivCards { cards } => {
            let names: Vec<String> = cards.iter().map(|c| c.to_string()).collect();
            format!("buy civilization card(s): {}", names.join(", "))
        }
        NetGameMove::DoneAcquiringCivCards => "done buying civilization cards".into(),
    }
}

/// Parse `<index>` or `<index> <tokens>` lines from stdin into SubmitMove.
fn submit_typed_moves(stdin: Res<StdinLines>, mut senders: Query<&mut MessageSender<SubmitMove>>) {
    let Ok(lines) = stdin.0.lock() else { return };
    while let Ok(line) = lines.try_recv() {
        let mut parts = line.split_whitespace();
        let Some(Ok(move_index)) = parts.next().map(str::parse::<usize>) else {
            println!("Could not parse {line:?} — type a move index, e.g. `0` or `0 2`.");
            continue;
        };
        let tokens = parts.next().and_then(|t| t.parse::<usize>().ok());
        for mut sender in senders.iter_mut() {
            sender.send::<ControlChannel>(SubmitMove {
                tokens,
                ..SubmitMove::index(move_index)
            });
            println!("→ submitted move {move_index}");
        }
    }
}
